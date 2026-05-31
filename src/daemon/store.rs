use std::path::Path;

use anyhow::Result;
use uuid::Uuid;

use crate::daemon::scanner::ParsedTodo;
use crate::todos::{Todo, TodoSource, modify_with_lock};

/// Merge note-parsed todos for `file` into the on-disk store.
///
/// - Preserves manual entries.
/// - Preserves NoteParsed entries for other files.
/// - For NoteParsed entries from `file`: matches by fingerprint to preserve
///   id + completed; otherwise generates a new Uuid.
pub async fn merge_file(
    cache_dir: &Path,
    file: &Path,
    parsed: Vec<ParsedTodo>,
) -> Result<Vec<Todo>> {
    let file = file.to_path_buf();
    modify_with_lock(cache_dir, move |list| {
        let (this_file_old, mut others): (Vec<Todo>, Vec<Todo>) =
            list.drain(..).partition(|t| match &t.source {
                TodoSource::NoteParsed { file: f, .. } => f == &file,
                _ => false,
            });

        let new_entries: Vec<Todo> = parsed
            .into_iter()
            .map(|p| {
                let prior = this_file_old
                    .iter()
                    .find(|old| match &old.source {
                        TodoSource::NoteParsed { fingerprint, .. } => {
                            *fingerprint == p.fingerprint
                        }
                        _ => false,
                    });
                let (id, completed) = match prior {
                    Some(old) => (old.id, p.completed),
                    None => (Uuid::new_v4(), p.completed),
                };
                Todo {
                    id,
                    text: p.text,
                    date: p.date,
                    completed,
                    source: TodoSource::NoteParsed {
                        file: p.file,
                        line: p.line,
                        fingerprint: p.fingerprint,
                    },
                }
            })
            .collect();

        others.extend(new_entries);
        *list = others;
    })
    .await
}

/// Drop all NoteParsed entries for `file` (used on file delete/rename-out).
pub async fn drop_file(cache_dir: &Path, file: &Path) -> Result<Vec<Todo>> {
    let file = file.to_path_buf();
    modify_with_lock(cache_dir, move |list| {
        list.retain(|t| match &t.source {
            TodoSource::NoteParsed { file: f, .. } => f != &file,
            _ => true,
        });
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use std::path::PathBuf;

    fn parsed(file: &str, text: &str, fingerprint: u64) -> ParsedTodo {
        ParsedTodo {
            text: text.to_string(),
            completed: false,
            date: None,
            file: PathBuf::from(file),
            line: 1,
            fingerprint,
        }
    }

    #[tokio::test]
    async fn merge_preserves_manual_entries() {
        let tmp = tempfile::tempdir().unwrap();
        let cache = tmp.path();

        crate::todos::save_todos(
            cache,
            &[Todo::new_manual(
                "manual one".to_string(),
                NaiveDate::from_ymd_opt(2026, 5, 31).unwrap(),
            )],
        )
        .await
        .unwrap();

        let final_list = merge_file(
            cache,
            Path::new("a.md"),
            vec![parsed("a.md", "from note", 1)],
        )
        .await
        .unwrap();

        assert_eq!(final_list.len(), 2);
        assert!(final_list.iter().any(|t| t.text == "manual one"
            && matches!(t.source, TodoSource::Manual)));
        assert!(final_list.iter().any(|t| t.text == "from note"
            && matches!(t.source, TodoSource::NoteParsed { .. })));
    }

    #[tokio::test]
    async fn merge_preserves_uuid_via_fingerprint() {
        let tmp = tempfile::tempdir().unwrap();
        let cache = tmp.path();

        // First pass.
        let first = merge_file(
            cache,
            Path::new("a.md"),
            vec![parsed("a.md", "ship it", 0xabc)],
        )
        .await
        .unwrap();
        let original_id = first[0].id;

        // Daemon rescans, same fingerprint, line moved.
        let mut p = parsed("a.md", "ship it", 0xabc);
        p.line = 42;
        let after = merge_file(cache, Path::new("a.md"), vec![p]).await.unwrap();

        assert_eq!(after.len(), 1);
        assert_eq!(after[0].id, original_id, "id preserved across rescan");
        if let TodoSource::NoteParsed { line, .. } = &after[0].source {
            assert_eq!(*line, 42, "line position updated");
        } else {
            panic!("expected NoteParsed");
        }
    }

    #[tokio::test]
    async fn merge_takes_note_as_source_of_truth_for_completion() {
        let tmp = tempfile::tempdir().unwrap();
        let cache = tmp.path();

        // First scan: note says checked.
        let mut p = parsed("a.md", "ship it", 0xabc);
        p.completed = true;
        merge_file(cache, Path::new("a.md"), vec![p]).await.unwrap();

        // Second scan: note now says unchecked. Should propagate.
        let p = parsed("a.md", "ship it", 0xabc); // completed = false by default
        let after = merge_file(cache, Path::new("a.md"), vec![p]).await.unwrap();
        assert_eq!(after.len(), 1);
        assert!(!after[0].completed, "note un-check should clear completion");
    }

    #[tokio::test]
    async fn merge_removes_deleted_lines() {
        let tmp = tempfile::tempdir().unwrap();
        let cache = tmp.path();

        merge_file(
            cache,
            Path::new("a.md"),
            vec![parsed("a.md", "first", 1), parsed("a.md", "second", 2)],
        )
        .await
        .unwrap();

        let after = merge_file(cache, Path::new("a.md"), vec![parsed("a.md", "first", 1)])
            .await
            .unwrap();

        assert_eq!(after.len(), 1);
        assert_eq!(after[0].text, "first");
    }

    #[tokio::test]
    async fn merge_does_not_touch_other_files() {
        let tmp = tempfile::tempdir().unwrap();
        let cache = tmp.path();

        merge_file(cache, Path::new("a.md"), vec![parsed("a.md", "from a", 1)])
            .await
            .unwrap();
        merge_file(cache, Path::new("b.md"), vec![parsed("b.md", "from b", 2)])
            .await
            .unwrap();

        // Rescan a.md with empty parsed -> should remove a's entries only.
        let after = merge_file(cache, Path::new("a.md"), vec![]).await.unwrap();
        assert_eq!(after.len(), 1);
        assert_eq!(after[0].text, "from b");
    }

    #[tokio::test]
    async fn drop_file_removes_all_for_path() {
        let tmp = tempfile::tempdir().unwrap();
        let cache = tmp.path();

        merge_file(
            cache,
            Path::new("a.md"),
            vec![parsed("a.md", "x", 1), parsed("a.md", "y", 2)],
        )
        .await
        .unwrap();

        let after = drop_file(cache, Path::new("a.md")).await.unwrap();
        assert!(after.is_empty());
    }
}
