use anyhow::Context;
use serde::{Deserialize, Serialize};
use slugify::slugify;

use crate::api::{ApiClient, branch::Branch, iteration::Iteration, story::comment::StoryComment};

pub mod comment;

#[derive(Deserialize, Serialize, Clone, PartialEq, Eq, Debug)]
pub struct Story {
    pub branches: Vec<Branch>,
    pub completed: bool,
    pub comments: Vec<StoryComment>,
    pub description: String,
    pub epic_id: Option<i32>,
    pub id: i32,
    pub iteration_id: Option<i32>,
    pub name: String,
    pub app_url: String,
}

#[derive(Deserialize)]
struct StorySearchResults {
    data: Vec<Story>,
    next: Option<String>,
}

impl ApiClient {
    pub async fn get_active_owned_stories(&self) -> anyhow::Result<Vec<Story>> {
        let query = format!("owner:{} !is:archived", self.mention_name);
        self.search_stories_all_pages(&query).await
    }

    pub(crate) async fn search_stories_all_pages(
        &self,
        query: &str,
    ) -> anyhow::Result<Vec<Story>> {
        let mut all = Vec::new();
        let mut next_path: Option<String> = None;

        loop {
            let response = match &next_path {
                None => {
                    let params = [
                        ("query", query),
                        ("page_size", "250"),
                        ("detail", "full"),
                    ];
                    self.get_with_query("search/stories", &params).await?
                }
                Some(path) => self.get_absolute_path(path).await?,
            };

            let page: StorySearchResults = response
                .json()
                .await
                .context("Failed to parse search/stories response")?;

            all.extend(page.data);
            match page.next {
                Some(n) => next_path = Some(n),
                None => break,
            }
        }

        Ok(all)
    }

    pub async fn update_story_description(
        &self,
        story_id: i32,
        new_description: String,
    ) -> anyhow::Result<()> {
        let body = serde_json::json!({
            "description": new_description,
        });

        let response = self
            .put_with_body(&format!("stories/{}", story_id), &body)
            .await?;

        // ignore the returned Story, we don't need and no need to parse the body of the response
        response.error_for_status()?;
        Ok(())
    }
}

impl Story {
    pub fn tmux_session_name(name: &str) -> String {
        let story_slug = slugify!(name);
        format!("scn--{}", story_slug)
    }

    pub fn get_file_name(&self) -> String {
        self.name.to_string()
    }
}

pub fn get_story_associated_iteration<'a>(
    iteration_id: Option<i32>,
    iterations: impl IntoIterator<Item = &'a Iteration>,
) -> Option<&'a Iteration> {
    let iteration_id = iteration_id?;
    iterations.into_iter().find(|it| it.id == iteration_id)
}
