use anyhow::{Context, Result};
use chrono::Utc;
use tokio::sync::mpsc;

use crate::{
    api::{iteration::Iteration, ApiClient},
    app::{
        model::{DataState, Model, UiState},
        msg::Msg,
        App,
    },
    cache::Cache,
    config::Config,
    get_api_key,
    get_user_id,
};

impl App {
    pub async fn init() -> Result<Self> {
        let config = Config::read()?;
        let mut cache = Cache::read(config.cache_dir.clone());

        let api_client = {
            let api_key = get_api_key().await?;
            let user_id = get_user_id(cache.user_id, &api_key).await?;
            ApiClient::new(api_key, user_id)
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
            },
            ui: UiState::default(),
            config: config.clone(),
            cache: cache.clone(),
        };

        let api_client_clone = api_client.clone();
        let saved_iteration = cache.current_iteration.clone();
        let saved_stories = cache.iteration_stories.clone();
        tokio::spawn(async move {
            let iteration = match get_current_iteration(saved_iteration, &api_client_clone).await {
                Ok(it) => {
                    let _ = sender.send(Msg::IterationLoaded(it.clone()));
                    it
                }
                Err(e) => {
                    let _ = sender.send(Msg::Error(e.to_string()));
                    return;
                }
            };

            if let Some(stories) = saved_stories {
                let _ = sender.send(Msg::StoriesLoaded { stories, from_cache: true });
            }

            match api_client_clone
                .get_owned_iteration_stories(&iteration)
                .await
            {
                Ok(stories) => {
                    let _ = sender.send(Msg::StoriesLoaded { stories, from_cache: false });
                }
                Err(e) => {
                    let _ = sender.send(Msg::Error(e.to_string()));
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
            cache,
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
