use anyhow::{Context, Result};
use chrono::Utc;
use tokio::sync::mpsc;

use crate::{
    api::{ApiClient, iteration::Iteration},
    app::{
        App,
        model::{DataState, Model, UiState},
        msg::Msg,
    },
    cache::Cache,
    config::Config,
    dummy,
    error::ErrorInfo,
    get_user_id,
};

impl App {
    pub async fn init() -> Result<Self> {
        let config = Config::read()?;
        let mut cache = Cache::read(config.cache_dir.clone());

        if dummy::is_enabled() {
            return Self::init_with_dummy_data(config, cache).await;
        }

        let api_client = {
            let user_id = get_user_id(cache.user_id, &config.api_token).await?;
            ApiClient::new(config.api_token.to_owned(), user_id)
        };

        cache.user_id = Some(api_client.user_id);
        cache.write()?;

        let (sender, receiver) = mpsc::unbounded_channel();
        let sender_clone = sender.clone();

        let model = Model {
            data: DataState {
                stories: cache.iteration_stories.clone().unwrap_or_default(),
                epics: vec![],
                current_iteration: cache.current_iteration.clone(),
                active_story: cache.active_story.clone(),
            },
            ui: UiState::default(),
            config: config.clone(),
            cache,
        };

        let api_client_clone = api_client.clone();
        let saved_iteration = model.cache.current_iteration.clone();
        let saved_stories = model.cache.iteration_stories.clone();
        tokio::spawn(async move {
            let iteration = match get_current_iteration(saved_iteration, &api_client_clone).await {
                Ok(it) => {
                    let _ = sender.send(Msg::IterationLoaded(it.clone()));
                    it
                }
                Err(e) => {
                    let info = ErrorInfo::new(
                        "Failed to fetch current iteration info".to_string(),
                        e.to_string(),
                    );

                    let _ = sender.send(Msg::Error(info));
                    return;
                }
            };

            if let Some(stories) = saved_stories {
                let _ = sender.send(Msg::StoriesLoaded {
                    stories,
                    from_cache: true,
                });
            }

            match api_client_clone
                .get_owned_iteration_stories(iteration.id)
                .await
            {
                Ok(stories) => {
                    let _ = sender.send(Msg::StoriesLoaded {
                        stories,
                        from_cache: false,
                    });
                }
                Err(e) => {
                    let info = ErrorInfo::new(
                        "Failed to get stories for current iteration".to_string(),
                        e.to_string(),
                    );

                    let _ = sender.send(Msg::Error(info));
                }
            }
        });

        Ok(Self {
            model,
            exit: false,
            receiver,
            sender: sender_clone,
            api_client,
            config,
        })
    }

    async fn init_with_dummy_data(config: Config, mut cache: Cache) -> Result<Self> {
        use uuid::Uuid;

        let dummy_user_id = Uuid::nil();
        let api_client = ApiClient::new(config.api_token.to_owned(), dummy_user_id);

        cache.user_id = Some(dummy_user_id);

        let (sender, receiver) = mpsc::unbounded_channel();
        let sender_clone = sender.clone();

        let iteration = dummy::iteration();
        let stories = dummy::stories();

        let model = Model {
            data: DataState {
                stories: stories.clone(),
                epics: vec![],
                current_iteration: Some(iteration.clone()),
                active_story: None,
            },
            ui: UiState::default(),
            config: config.clone(),
            cache,
        };

        // Send messages so UI updates as if data loaded normally
        let _ = sender.send(Msg::IterationLoaded(iteration));
        let _ = sender.send(Msg::StoriesLoaded {
            stories,
            from_cache: false,
        });

        Ok(Self {
            model,
            exit: false,
            receiver,
            sender: sender_clone,
            api_client,
            config,
        })
    }
}

async fn get_current_iteration(
    saved_iteration: Option<Iteration>,
    api_client: &ApiClient,
) -> anyhow::Result<Iteration> {
    // NOTE: using UTC here could cause timezone issues
    let today = Utc::now().date_naive();
    if let Some(iteration) = saved_iteration
        && iteration.start_date <= today
        && iteration.end_date >= today
    {
        Ok(iteration)
    } else {
        api_client
            .get_current_iteration()
            .await
            .context("Failed to get the current iteration")
    }
}
