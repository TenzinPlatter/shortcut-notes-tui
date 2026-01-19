use anyhow::Context;
use serde::Deserialize;

use crate::api::{ApiClient, branch::Branch, story::comment::StoryComment};

pub mod comment;
pub mod view;

#[derive(Deserialize)]
#[allow(dead_code)] // TODO: remove, just while implementing
pub struct Story {
    pub branches: Vec<Branch>,
    pub completed: bool,
    pub comments: Vec<StoryComment>,
    pub description: String,
    pub epic_id: Option<i32>,
    pub id: i32,
    pub iteration_id: Option<i32>,
    pub name: String,
}

#[derive(Deserialize)]
pub struct StorySlim {
    pub id: i32,
}

impl ApiClient {
    pub async fn get_owned_stories(&self) -> anyhow::Result<Vec<Story>> {
        let body = serde_json::json!({
            "archived": false,
            "owner_ids": [self.user_id],
        });

        let stories_slim = {
            let response = self.post_with_body("stories/search", &body).await?;
            response
                .json::<Vec<StorySlim>>()
                .await
                .context("Failed to fetch owned stories from API")?
        };

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
