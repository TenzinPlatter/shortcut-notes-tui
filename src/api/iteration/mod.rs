use anyhow::Context;
use chrono::{NaiveDate, Utc};
use futures::future::try_join_all;
use serde::Deserialize;

use crate::api::{
    ApiClient,
    story::{Story, StorySlim},
};

#[derive(Deserialize)]
pub struct Iteration {
    id: i32,
    name: String,
    description: String,
}

#[derive(Deserialize)]
pub struct IterationSlim {
    id: i32,
    start_date: NaiveDate,
    end_date: NaiveDate,
}

impl ApiClient {
    pub async fn get_current_iteration(&self) -> anyhow::Result<Iteration> {
        let response = self.get("iterations").await?;
        let iterations_slim = response.json::<Vec<IterationSlim>>().await?;
        let today = Utc::now().date_naive();
        let current_iteration_id = iterations_slim
            .iter()
            // will return first that matches condition
            .find(|it| it.start_date <= today && it.end_date >= today)
            .map(|it| it.id);

        if let Some(id) = current_iteration_id {
            let response = self.get(&format!("iterations/{id}")).await?;
            Ok(response.json::<Iteration>().await?)
        } else {
            // TODO: handle this, return an option
            anyhow::bail!("Couldn't find a current iteration");
        }
    }

    pub async fn get_owned_iteration_stories(&self, iteration: &Iteration) -> anyhow::Result<Vec<Story>> {
        let response = self
            .get(&format!("iterations/{}/stories", iteration.id))
            .await?;
        let stories_slim = response.json::<Vec<StorySlim>>().await?;
        let slim_owned: Vec<_> = stories_slim.iter().filter(|s| s.owner_ids.contains(&self.user_id)).collect();

        let stories = {
            let len = slim_owned.len().min(5);
            let futures = slim_owned.into_iter().take(len).map(|slim| async move {
                let query = format!("stories/{}", slim.id);
                let response = self.get(&query).await?;
                response
                    .json::<Story>()
                    .await
                    .context("Failed to parse as Story")
            });
            
            try_join_all(futures).await?
        };

        Ok(stories)
    }
}
