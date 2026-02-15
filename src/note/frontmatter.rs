use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use slugify::slugify;

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
    pub fn new(
        story_id: i32,
        story_name: String,
        story_app_url: String,
        iteration_app_url: Option<String>,
    ) -> Self {
        let slug = slugify!(&story_name);

        Self {
            slug_id: slug,

            story_id: format!("sc-{}", story_id),
            story_name,
            story_link: story_app_url,

            iteration_link: iteration_app_url,

            epic_link: None,

            created: crate::time::today(),
            note_type: NoteType::General,
            tags: Vec::new(),
            aliases: Vec::new(),
        }
    }

    pub fn to_yaml_string(&self) -> anyhow::Result<String> {
        Ok(serde_yaml::to_string(self)?)
    }
}
