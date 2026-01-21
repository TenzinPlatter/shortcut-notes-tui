use anyhow::Context;
use ratatui::{style::Style, text::Line, widgets::ListItem};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    api::{ApiClient, branch::Branch, story::comment::StoryComment},
    view::list::ExpandableListItem,
};

pub mod comment;

#[derive(Deserialize, Serialize, Clone, PartialEq, Eq)]
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
}

impl ExpandableListItem for Story {
    fn as_list_item(&self, expanded: bool) -> ListItem<'static> {
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
                Line::from("  Press <Enter> to view description").style(Style::default().italic()),
            )
        }

        text.push(Line::from(""));

        ListItem::new(text)
    }
}
