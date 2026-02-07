use anyhow::Context;
use ratatui::{style::Style, text::Line, widgets::ListItem};
use serde::{Deserialize, Serialize};
use slugify::slugify;
use uuid::Uuid;

use crate::{
    api::{ApiClient, branch::Branch, story::comment::StoryComment},
};

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
pub struct StorySlim {
    pub id: i32,
    pub owner_ids: Vec<Uuid>,
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

    pub async fn update_story_description(
        &self,
        story: &Story,
        new_description: String,
    ) -> anyhow::Result<()> {
        let body = serde_json::json!({
            "description": new_description,
        });

        let response = self
            .put_with_body(&format!("stories/{}", story.id), &body)
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

    pub fn into_list_item(&self, expanded: bool, selected: bool) -> ListItem<'static> {
        let mut text = vec![
            Line::from(self.name.to_string()),
            Line::from("Description:"),
        ];

        if expanded {
            for line in self.description.lines() {
                text.push(Line::from(format!("  {}", line)));
            }
        } else {
            text.push(
                Line::from("  Press <Space> to view description")
                .style(Style::default().italic()),
            )
        }

        text.push(Line::from(""));

        let style = if selected {
            Style::default().reversed()
        } else {
            Style::default()
        };

        ListItem::new(text).style(style)
    }

    pub fn get_file_name(&self) -> String {
        self.name.to_string()
    }
}
