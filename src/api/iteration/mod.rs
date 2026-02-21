use anyhow::Context;
use chrono::NaiveDate;
use futures::future::{join_all, try_join_all};
use serde::{Deserialize, Serialize};

use crate::{
    api::{
        ApiClient,
        story::{Story, StorySlim},
    },
    custom_list::LinearListItem,
    dbg_file,
};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Iteration {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub app_url: String,
}

#[derive(Deserialize)]
pub struct IterationSlim {
    id: i32,
    start_date: NaiveDate,
    end_date: NaiveDate,
}

impl LinearListItem for Iteration {
    fn id(&self) -> i32 { self.id }
    fn label(&self) -> &str { &self.name }
}

impl ApiClient {
    pub async fn get_current_iterations(&self) -> anyhow::Result<Vec<Iteration>> {
        let response = self.get("iterations").await?;
        let iterations_slim = response.json::<Vec<IterationSlim>>().await?;
        let today = crate::time::today();
        let current_iteration_ids: Vec<_> = iterations_slim
            .iter()
            .filter(|it| it.start_date <= today && it.end_date >= today)
            .map(|it| it.id)
            .collect();

        let iterations = join_all(current_iteration_ids.iter().map(|id| async move {
            let response = self.get(&format!("iterations/{id}")).await?;
            // add the context to cast response to an anyhow error, but we will filter out errors
            // so don't need a real message
            response.json::<Iteration>().await.context("")
        }))
        .await
        .into_iter()
        .filter_map(|res| res.ok())
        .collect();

        Ok(iterations)
    }

    pub async fn get_all_iterations(&self) -> anyhow::Result<Vec<Iteration>> {
        let response = self.get("iterations").await?;
        let iterations_slim = response.json::<Vec<IterationSlim>>().await?;

        let iterations = join_all(iterations_slim.iter().map(|slim| async move {
            let response = self.get(&format!("iterations/{}", slim.id)).await?;
            response.json::<Iteration>().await.context("")
        }))
        .await
        .into_iter()
        .filter_map(|res| res.ok())
        .collect();

        Ok(iterations)
    }

    pub async fn get_owned_iteration_stories(
        &self,
        iteration_ids: Vec<i32>,
    ) -> anyhow::Result<Vec<Story>> {
        let iteration_stories = join_all(
            iteration_ids
                .iter()
                .map(|id| async { self.get_owned_single_iteration_stories(*id).await }),
        )
        .await
        .into_iter()
        .filter_map(|res| match res {
            Ok(stories) => Some(stories),
            Err(e) => {
                dbg_file!("Failed to fetch story with error: {}", e);
                None
            }
        })
        .flatten()
        .collect();

        Ok(iteration_stories)
    }

    async fn get_owned_single_iteration_stories(
        &self,
        iteration_id: i32,
    ) -> anyhow::Result<Vec<Story>> {
        let response = self
            .get(&format!("iterations/{}/stories", iteration_id))
            .await?;
        let stories_slim = response.json::<Vec<StorySlim>>().await?;
        let slim_owned: Vec<_> = stories_slim
            .iter()
            .filter(|s| s.owner_ids.contains(&self.user_id))
            .collect();

        let stories = {
            let len = slim_owned.len();
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
