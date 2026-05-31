use std::path::PathBuf;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Todo {
    pub id: Uuid,
    pub text: String,
    pub date: NaiveDate,
    pub completed: bool,
}

impl Todo {
    pub fn new(text: String, date: NaiveDate) -> Self {
        Self {
            id: Uuid::new_v4(),
            text,
            date,
            completed: false,
        }
    }
}

pub async fn load_todos(cache_dir: &PathBuf) -> Vec<Todo> {
    let path = cache_dir.join("todos.json");
    match tokio::fs::read_to_string(&path).await {
        Ok(content) => {
            let todos: Vec<Todo> = serde_json::from_str(&content).unwrap_or_default();
            todos.into_iter().filter(|t| !t.completed).collect()
        }
        Err(_) => Vec::new(),
    }
}

pub async fn save_todos(cache_dir: &PathBuf, todos: &[Todo]) -> anyhow::Result<()> {
    let path = cache_dir.join("todos.json");
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    let content = serde_json::to_string_pretty(todos)?;
    tokio::fs::write(&path, content).await?;
    Ok(())
}
