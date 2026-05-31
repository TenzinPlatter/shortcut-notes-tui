use std::path::PathBuf;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Todo {
    pub id: Uuid,
    pub text: String,
    pub date: Option<NaiveDate>,
    pub completed: bool,
    #[serde(default)]
    pub source: TodoSource,
}

#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum TodoSource {
    #[default]
    Manual,
    NoteParsed {
        file: PathBuf,
        line: u32,
        fingerprint: u64,
    },
}

impl Todo {
    pub fn new_manual(text: String, date: NaiveDate) -> Self {
        Self {
            id: Uuid::new_v4(),
            text,
            date: Some(date),
            completed: false,
            source: TodoSource::Manual,
        }
    }
}

pub async fn load_todos(cache_dir: &PathBuf) -> Vec<Todo> {
    let path = cache_dir.join("todos.json");
    match tokio::fs::read_to_string(&path).await {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn deserializes_legacy_todo_without_source_field() {
        let json = r#"{
            "id": "00000000-0000-0000-0000-000000000001",
            "text": "legacy",
            "date": "2026-05-31",
            "completed": false
        }"#;
        let todo: Todo = serde_json::from_str(json).unwrap();
        assert_eq!(todo.text, "legacy");
        assert_eq!(todo.date, Some(chrono::NaiveDate::from_ymd_opt(2026, 5, 31).unwrap()));
        assert!(matches!(todo.source, TodoSource::Manual));
    }

    #[test]
    fn deserializes_todo_without_date() {
        let json = r#"{
            "id": "00000000-0000-0000-0000-000000000002",
            "text": "no date",
            "date": null,
            "completed": false
        }"#;
        let todo: Todo = serde_json::from_str(json).unwrap();
        assert_eq!(todo.date, None);
    }

    #[test]
    fn round_trips_note_parsed_source() {
        let todo = Todo {
            id: uuid::Uuid::nil(),
            text: "scan".to_string(),
            date: None,
            completed: false,
            source: TodoSource::NoteParsed {
                file: PathBuf::from("alpha.md"),
                line: 12,
                fingerprint: 0xdead_beef,
            },
        };
        let json = serde_json::to_string(&todo).unwrap();
        let back: Todo = serde_json::from_str(&json).unwrap();
        assert!(matches!(
            back.source,
            TodoSource::NoteParsed { line: 12, fingerprint: 0xdead_beef, .. }
        ));
    }
}
