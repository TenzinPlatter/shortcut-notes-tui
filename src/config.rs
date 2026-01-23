use anyhow::Context;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Default, Clone)]
pub struct Config {
    pub notes_dir: String,
    pub cache_dir: Option<String>,
}

impl Config {
    pub fn read() -> anyhow::Result<Self> {
        confy::load("shortcut-notes", Some("config")).context("Failed to read config")
    }

    pub fn write(&self) -> anyhow::Result<()> {
        confy::store("shortcut-notes", Some("config"), self)?;
        Ok(())
    }
}
