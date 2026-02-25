use std::path::{Path, PathBuf};

use crossterm::event::{KeyCode, KeyEvent};

use crate::{
    app::{cmd::Cmd, msg::NotesListMsg},
    navkey,
};

pub use crate::app::model::NotesListState;

/// Scans one subdirectory of the notes directory and returns `.md` files sorted descending.
fn scan_subdir(notes_dir: &Path, subdir: &str) -> Vec<PathBuf> {
    let dir = notes_dir.join(subdir);
    let mut notes = Vec::new();

    let entries = match std::fs::read_dir(&dir) {
        Ok(entries) => entries,
        Err(_) => return notes,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }
        notes.push(path);
    }

    notes.sort_by(|a, b| b.file_name().cmp(&a.file_name()));
    notes
}

/// Scans all note subdirectories and returns per-category vecs.
pub fn scan_notes(notes_dir: &Path) -> (Vec<PathBuf>, Vec<PathBuf>, Vec<PathBuf>, Vec<PathBuf>, Vec<PathBuf>) {
    let daily = scan_subdir(notes_dir, "daily");
    let stories = scan_subdir(notes_dir, "stories");
    let iterations = scan_subdir(notes_dir, "iterations");
    let epics = scan_subdir(notes_dir, "epics");
    let scratch = scan_subdir(notes_dir, "scratch");
    (daily, stories, iterations, epics, scratch)
}

/// Returns a flat list of all notes in display order.
fn all_notes(state: &NotesListState) -> Vec<&PathBuf> {
    state
        .daily_notes
        .iter()
        .chain(state.story_notes.iter())
        .chain(state.iteration_notes.iter())
        .chain(state.epic_notes.iter())
        .chain(state.scratch_notes.iter())
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
