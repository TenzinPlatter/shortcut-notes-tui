use anyhow::{Context, Result};
use chrono::Utc;
use tokio::sync::mpsc;

use crate::{
    api::{iteration::Iteration, ApiClient}, app::{events::AppEvent, App}, cache::Cache, config::Config, get_api_key, get_user_id
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

        // Create channel for background tasks to communicate with main app
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        let event_tx_clone = event_tx.clone();

        // Start with a loading view
        let view = Self::get_loading_view_iteration();

        let api_client_clone = api_client.clone();
        let saved_iteration = cache.current_iteration.clone();
        let saved_stories = cache.iteration_stories.clone();
        tokio::spawn(async move {
            let iteration = match get_current_iteration(saved_iteration, &api_client_clone).await {
                Ok(it) => {
                    let _ = event_tx.send(AppEvent::IterationLoaded(it.clone()));
                    it
                }
                Err(e) => {
                    let _ = event_tx.send(AppEvent::UnexpectedError(e));
                    return;
                }
            };

            if let Some(stories) = saved_stories {
                // TODO: add something visual to show that we are still fetching stories from api
                let _ = event_tx.send(AppEvent::StoriesLoaded((stories, true)));
            }

            match api_client_clone
                .get_owned_iteration_stories(&iteration)
                .await
            {
                Ok(stories) => {
                    let _ = event_tx.send(AppEvent::StoriesLoaded((stories, false)));
                }
                Err(e) => {
                    let _ = event_tx.send(AppEvent::UnexpectedError(e));
                }
            }
        });

        Ok(Self {
            config,
            view,
            exit: false,
            api_client,
            reciever: event_rx,
            sender: event_tx_clone,
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
