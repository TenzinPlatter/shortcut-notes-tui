use anyhow::{Context, Result};
use chrono::Utc;
use tokio::sync::mpsc;

use crate::{
    api::{ApiClient, iteration::Iteration},
    app::{App, events::AppEvent},
    config::Config,
    get_api_key, get_user_id,
};

impl App {
    pub async fn init() -> Result<Self> {
        let mut config = Config::read()?;

        let api_client = {
            let api_key = get_api_key().await?;
            let user_id = get_user_id(config.user_id, &api_key).await?;
            ApiClient::new(api_key, user_id)
        };

        config.user_id = Some(api_client.user_id);
        config.write()?;

        // Create channel for background tasks to communicate with main app
        let (event_tx, event_rx) = mpsc::unbounded_channel();

        // Start with a loading view
        let view = Self::get_loading_view_iteration();

        let api_client_clone = api_client.clone();
        let saved_iteration = config.current_iteration.clone();
        let saved_stories = config.iteration_stories.clone();
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
                let _ = event_tx.send(AppEvent::StoriesLoaded(stories));
            }

            match api_client_clone
                .get_owned_iteration_stories(&iteration)
                .await
            {
                Ok(stories) => {
                    let _ = event_tx.send(AppEvent::StoriesLoaded(stories));
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
            event_rx,
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
