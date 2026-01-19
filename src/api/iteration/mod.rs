use chrono::{DateTime, FixedOffset, Utc};
use serde::Deserialize;

use crate::api::{story::{Story, StorySlim}, ApiClient};

#[derive(Deserialize)]
pub struct Iteration {
    id: i32,
    name: String,
    description: String,
}

#[derive(Deserialize)]
pub struct IterationSlim {
    id: i32,
    start_date: DateTime<FixedOffset>,
    end_date: DateTime<FixedOffset>,
}

impl ApiClient {
    pub async fn get_current_iteration(&self) -> anyhow::Result<Iteration> {
        let response = self.get("iterations").await?;
        let iterations_slim = response.json::<Vec<IterationSlim>>().await?;
        let now = Utc::now();
        let current_iteration_id = iterations_slim
            .iter()
            // will return first that matches condition
            .find(|it| it.start_date <= now && it.end_date >= now)
            .map(|it| it.id);

        if let Some(id) = current_iteration_id {
            let response = self.get(&format!("iterations/{id}")).await?;
            Ok(response.json::<Iteration>().await?)
        } else {
            // TODO: handle this, return an option
            anyhow::bail!("Couldn't find a current iteration");
        }
    }

    pub async fn get_iteration_stories(&self, iteration: &Iteration) -> anyhow::Result<Vec<Story>> {
        let response = self.get(&format!("iterations/{}/stories", iteration.id)).await?;
        let stories_slim = response.json::<Vec<StorySlim>>().await?;

        let stories = {
            let mut stories = Vec::with_capacity(stories_slim.len());
            for slim in stories_slim.into_iter() {
                let query = format!("stories/{}", slim.id);
                let response = self.get(&query).await?;
                let story = response.json::<Story>().await?;
                stories.push(story);
            }
            stories
        };

        Ok(stories)
    }
}
