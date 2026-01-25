use chrono::{Datelike, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use slugify::slugify;

use crate::api::{iteration::Iteration, story::Story};

#[derive(Debug, Deserialize, Serialize)]
pub enum NoteType {
    Meeting,
    Idea,
    Todo,
    General,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Frontmatter {
    /// this is/will be used as the filename
    #[serde(skip_serializing, skip_deserializing)]
    pub slug_id: String,

    /// Story fields
    story_id: String, // e.g. sc-12345
    story_link: String,
    story_name: String,

    /// Iteration fields
    #[serde(rename = "iteration")]
    iteration_link: Option<String>,

    /// Epic fields
    #[serde(rename = "epic")]
    epic_link: Option<String>,

    /// Note fields
    created: NaiveDate,
    #[serde(rename = "type")]
    note_type: NoteType,
    tags: Vec<String>,

    /// Obisidian fields
    aliases: Vec<String>,
}

impl Frontmatter {
    // TODO: epic
    pub fn new(story: &Story, current_iteration: Option<&Iteration>) -> Self {
        let slug = slugify!(&story.name);
        let now = Utc::now();
        let slug_id = format!("{}-{}", now.day(), &slug);
        let iteration_link = current_iteration.map(|it| it.app_url.clone());

        Self {
            slug_id,

            story_id: format!("sc-{}", story.id),
            story_name: story.name.clone(),
            story_link: story.app_url.clone(),

            iteration_link,

            epic_link: None,

            created: now.date_naive(),
            note_type: NoteType::General,
            tags: Vec::new(),
            aliases: Vec::new(),
        }
    }

    pub fn to_yaml_string(&self) -> anyhow::Result<String> {
        Ok(serde_yaml::to_string(self)?)
    }
}
