use std::path::{Path, PathBuf};

use crossterm::event::{KeyCode, KeyEvent};

use crate::{
    app::{cmd::Cmd, msg::NotesListMsg},
    navkey,
};

pub use crate::app::model::NotesListState;

/// Scans the notes directory and classifies files into daily notes and other notes.
/// Daily notes match the pattern `daily-*.md`.
/// Both lists are sorted by filename descending (newest date first).
pub fn scan_notes(notes_dir: &Path) -> (Vec<PathBuf>, Vec<PathBuf>) {
    let mut daily = Vec::new();
    let mut other = Vec::new();

    let entries = match std::fs::read_dir(notes_dir) {
        Ok(entries) => entries,
        Err(_) => return (daily, other),
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let ext = path.extension().and_then(|e| e.to_str());
        if ext != Some("md") {
            continue;
        }

        let filename = match path.file_name().and_then(|f| f.to_str()) {
            Some(f) => f.to_string(),
            None => continue,
        };

        if filename.starts_with("daily-") {
            daily.push(path);
        } else {
            other.push(path);
        }
    }

    // Sort descending by filename (newest dates first)
    daily.sort_by(|a, b| b.file_name().cmp(&a.file_name()));
    other.sort_by(|a, b| b.file_name().cmp(&a.file_name()));

    (daily, other)
}

/// Returns a flat list of all notes: daily notes first, then other notes.
fn all_notes(state: &NotesListState) -> Vec<&PathBuf> {
    state
        .daily_notes
        .iter()
        .chain(state.other_notes.iter())
        .collect()
}

pub fn update(state: &mut NotesListState, msg: NotesListMsg) -> Vec<Cmd> {
    match msg {
        NotesListMsg::FocusNext => {
            let notes = all_notes(state);
            if notes.is_empty() {
                return vec![Cmd::None];
            }

            let current_idx = state
                .selected_path
                .as_ref()
                .and_then(|sel| notes.iter().position(|p| *p == sel));

            let next_idx = match current_idx {
                Some(idx) => (idx + 1) % notes.len(),
                None => 0,
            };

            state.selected_path = Some(notes[next_idx].clone());
            vec![Cmd::None]
        }

        NotesListMsg::FocusPrev => {
            let notes = all_notes(state);
            if notes.is_empty() {
                return vec![Cmd::None];
            }

            let current_idx = state
                .selected_path
                .as_ref()
                .and_then(|sel| notes.iter().position(|p| *p == sel));

            let prev_idx = match current_idx {
                Some(0) => notes.len() - 1,
                Some(idx) => idx - 1,
                None => notes.len() - 1,
            };

            state.selected_path = Some(notes[prev_idx].clone());
            vec![Cmd::None]
        }

        NotesListMsg::OpenNote => {
            if let Some(ref path) = state.selected_path {
                vec![Cmd::OpenDailyNote {
                    path: path.clone(),
                }]
            } else {
                vec![Cmd::None]
            }
        }
    }
}

pub fn key_to_msg(key: KeyEvent) -> Option<NotesListMsg> {
    match key.code {
        navkey!(down) => Some(NotesListMsg::FocusNext),
        navkey!(up) => Some(NotesListMsg::FocusPrev),
        KeyCode::Enter => Some(NotesListMsg::OpenNote),
        _ => None,
    }
}
