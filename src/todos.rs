use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use chrono::NaiveDate;
use fs2::FileExt;
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

const TODOS_FILE: &str = "todos.json";
const TODOS_LOCK_FILE: &str = "todos.json.lock";

fn lock_path(cache_dir: &Path) -> PathBuf {
    cache_dir.join(TODOS_LOCK_FILE)
}

fn todos_path(cache_dir: &Path) -> PathBuf {
    cache_dir.join(TODOS_FILE)
}

fn open_lock(cache_dir: &Path) -> anyhow::Result<std::fs::File> {
    if !cache_dir.exists() {
        std::fs::create_dir_all(cache_dir)?;
    }
    let path = lock_path(cache_dir);
    let file = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .truncate(false)
        .open(&path)?;
    Ok(file)
}

fn read_todos_inner(cache_dir: &Path) -> Vec<Todo> {
    let path = todos_path(cache_dir);
    let Ok(mut f) = std::fs::File::open(&path) else {
        return Vec::new();
    };
    let mut buf = String::new();
    if f.read_to_string(&mut buf).is_err() {
        return Vec::new();
    }
    serde_json::from_str(&buf).unwrap_or_default()
}

fn write_todos_inner(cache_dir: &Path, todos: &[Todo]) -> anyhow::Result<()> {
    std::fs::create_dir_all(cache_dir)?;
    let path = todos_path(cache_dir);
    let mut tmp = tempfile::NamedTempFile::new_in(cache_dir)?;
    let content = serde_json::to_string_pretty(todos)?;
    tmp.write_all(content.as_bytes())?;
    tmp.persist(&path)?;
    Ok(())
}

pub async fn load_todos(cache_dir: &Path) -> Vec<Todo> {
    let cache_dir = cache_dir.to_path_buf();
    tokio::task::spawn_blocking(move || read_todos_inner(&cache_dir))
        .await
        .unwrap_or_default()
}

pub async fn save_todos(cache_dir: &Path, todos: &[Todo]) -> anyhow::Result<()> {
    let cache_dir = cache_dir.to_path_buf();
    let todos = todos.to_vec();
    tokio::task::spawn_blocking(move || {
        let lock = open_lock(&cache_dir)?;
        FileExt::lock_exclusive(&lock)?;
        let res = write_todos_inner(&cache_dir, &todos);
        let _ = FileExt::unlock(&lock);
        res
    })
    .await?
}

/// Read-modify-write under flock. The closure receives a mutable Vec<Todo>
/// it can edit in place; on return the new list is written atomically.
pub async fn modify_with_lock<F>(cache_dir: &Path, f: F) -> anyhow::Result<Vec<Todo>>
where
    F: FnOnce(&mut Vec<Todo>) + Send + 'static,
{
    let cache_dir = cache_dir.to_path_buf();
    tokio::task::spawn_blocking(move || {
        let lock = open_lock(&cache_dir)?;
        FileExt::lock_exclusive(&lock)?;
        let mut list = read_todos_inner(&cache_dir);
        f(&mut list);
        write_todos_inner(&cache_dir, &list)?;
        let _ = FileExt::unlock(&lock);
        Ok(list)
    })
    .await?
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

    #[test]
    fn concurrent_writes_do_not_lose_updates() {
        use std::thread;

        let tmp = tempfile::tempdir().unwrap();
        let cache_dir = tmp.path().to_path_buf();
        let rt = tokio::runtime::Runtime::new().unwrap();

        // Pre-seed an empty list so the file + lock exist.
        rt.block_on(save_todos(&cache_dir, &[])).unwrap();

        let a = cache_dir.clone();
        let b = cache_dir.clone();

        let h1 = thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            for i in 0..20 {
                rt.block_on(modify_with_lock(&a, move |list| {
                    list.push(Todo::new_manual(
                        format!("a{}", i),
                        chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                    ));
                })).unwrap();
            }
        });
        let h2 = thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            for i in 0..20 {
                rt.block_on(modify_with_lock(&b, move |list| {
                    list.push(Todo::new_manual(
                        format!("b{}", i),
                        chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                    ));
                })).unwrap();
            }
        });
        h1.join().unwrap();
        h2.join().unwrap();

        let final_todos = rt.block_on(load_todos(&cache_dir));
        assert_eq!(final_todos.len(), 40, "lost updates detected");
    }
}
