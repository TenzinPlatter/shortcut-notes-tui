use anyhow::Context;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::api::{iteration::Iteration, story::Story};

#[derive(Deserialize, Serialize, Default, Clone)]
pub struct Config {
    pub current_iteration: Option<Iteration>,
    pub iteration_stories: Option<Vec<Story>>,
    pub user_id: Option<Uuid>,
}

impl Config {
    pub fn read() -> anyhow::Result<Self> {
        confy::load("shortcut-notes-tui", Some("config")).context("Failed to read config")
    }

    pub fn write(&self) -> anyhow::Result<()> {
        confy::store("shortcut-notes-tui", Some("config"), self)?;
        Ok(())
    }
}
