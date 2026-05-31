use std::path::{Path, PathBuf};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

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

/// Per-category vecs returned by `scan_notes`: daily, stories, iterations, epics, scratch.
type NotesScan = (
    Vec<PathBuf>,
    Vec<PathBuf>,
    Vec<PathBuf>,
    Vec<PathBuf>,
    Vec<PathBuf>,
);

/// Scans all note subdirectories and returns per-category vecs.
pub fn scan_notes(notes_dir: &Path) -> NotesScan {
    let daily = scan_subdir(notes_dir, "daily");
    let stories = scan_subdir(notes_dir, "stories");
    let iterations = scan_subdir(notes_dir, "iterations");
    let epics = scan_subdir(notes_dir, "epics");
    let scratch = scan_subdir(notes_dir, "scratch");
    (daily, stories, iterations, epics, scratch)
}

/// Fixed section order: 0=daily, 1=story, 2=iteration, 3=epic, 4=scratch.
fn section_notes(state: &NotesListState) -> [&Vec<PathBuf>; 5] {
    [
        &state.daily_notes,
        &state.story_notes,
        &state.iteration_notes,
        &state.epic_notes,
        &state.scratch_notes,
    ]
}

/// Returns the section index (0–4) that the given path belongs to.
fn section_of(state: &NotesListState, path: &PathBuf) -> Option<usize> {
    section_notes(state)
        .iter()
        .enumerate()
        .find(|(_, notes)| notes.contains(path))
        .map(|(i, _)| i)
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

        NotesListMsg::FocusSectionNext | NotesListMsg::FocusSectionPrev => {
            let sections = section_notes(state);
            let non_empty: Vec<usize> = (0..5).filter(|&i| !sections[i].is_empty()).collect();
            if non_empty.is_empty() {
                return vec![Cmd::None];
            }

            // Save current selection for current section
            if let Some(ref sel) = state.selected_path.clone()
                && let Some(cur_idx) = section_of(state, sel)
            {
                state.section_selections.insert(cur_idx, sel.clone());
            }

            // Find current section index within non_empty list
            let cur_section = state.selected_path.as_ref()
                .and_then(|sel| section_of(state, sel));
            let cur_pos = cur_section
                .and_then(|s| non_empty.iter().position(|&i| i == s))
                .unwrap_or(0);

            let next_pos = if matches!(msg, NotesListMsg::FocusSectionNext) {
                (cur_pos + 1) % non_empty.len()
            } else {
                (cur_pos + non_empty.len() - 1) % non_empty.len()
            };
            let target = non_empty[next_pos];

            // Restore saved selection for target section, or default to first item
            let sections = section_notes(state);
            state.selected_path = state
                .section_selections
                .get(&target)
                .filter(|p| sections[target].contains(p))
                .cloned()
                .or_else(|| sections[target].first().cloned());

            vec![Cmd::None]
        }

        NotesListMsg::OpenNote => {
            if let Some(ref path) = state.selected_path {
                vec![Cmd::OpenDailyNote { path: path.clone() }]
            } else {
                vec![Cmd::None]
            }
        }
    }
}

pub fn key_to_msg(key: KeyEvent) -> Option<NotesListMsg> {
    match key.code {
        KeyCode::Char('j') if key.modifiers == KeyModifiers::CONTROL => {
            Some(NotesListMsg::FocusSectionNext)
        }
        KeyCode::Char('k') if key.modifiers == KeyModifiers::CONTROL => {
            Some(NotesListMsg::FocusSectionPrev)
        }
        navkey!(down) => Some(NotesListMsg::FocusNext),
        navkey!(up) => Some(NotesListMsg::FocusPrev),
        KeyCode::Enter => Some(NotesListMsg::OpenNote),
        _ => None,
    }
}
