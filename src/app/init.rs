use anyhow::Result;
use tokio::{sync::mpsc::{self, UnboundedSender}, task::JoinHandle};
use uuid::Uuid;

use crate::{
    api::ApiClient,
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

        let mut model = Model::from_cache_and_config(cache, config.clone());

        let handles = fetch_info_from_api(api_client.clone(), sender).await;
        model.data.async_handles.extend(handles);

        Ok(App {
            model,
            exit: false,
            receiver,
            sender: sender_clone,
            api_client,
            config,
        })
    }

    async fn init_with_dummy_data(config: Config, mut cache: Cache) -> Result<Self> {
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
                epics: Vec::new(),
                current_iterations: Some(vec![iteration.clone()]),
                active_story: None,
                async_handles: Vec::new(),
                iterations: vec![iteration.clone()],
            },
            ui: UiState::default(),
            config: config.clone(),
            cache,
        };

        // Send messages so UI updates as if data loaded normally
        let _ = sender.send(Msg::IterationsLoaded(vec![iteration]));
        let _ = sender.send(Msg::StoriesLoaded {
            stories,
            from_cache: false,
        });

        Ok(App {
            model,
            exit: false,
            receiver,
            sender: sender_clone,
            api_client,
            config,
        })
    }
}

async fn fetch_info_from_api(api_client: ApiClient, sender: UnboundedSender<Msg>) -> Vec<JoinHandle<()>> {
    let current_iteration_handle = tokio::spawn(async move {
        match api_client.get_current_iterations().await {
            Ok(iterations) => {
                let _ = sender.send(Msg::IterationsLoaded(iterations));
            }
            Err(e) => {
                let info = ErrorInfo::new(
                    "Failed to fetch current iteration info".to_string(),
                    e.to_string(),
                );

                let _ = sender.send(Msg::Error(info));
            }
        };
    });

    vec![current_iteration_handle]
}

