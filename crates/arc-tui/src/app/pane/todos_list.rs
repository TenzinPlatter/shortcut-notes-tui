use std::collections::BTreeMap;

use chrono::NaiveDate;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use uuid::Uuid;

use crate::{
    app::{cmd::Cmd, msg::TodosListMsg},
    todos::Todo,
};

pub use crate::app::model::TodosListState;

#[derive(Clone, Debug)]
pub enum SectionKind {
    Overdue,
    Today,
    Tomorrow,
    Future(NaiveDate),
    NoDate,
}

impl SectionKind {
    pub const OVERDUE: &'static str = "Overdue";
    pub const TODAY: &'static str = "Today";
    pub const TOMORROW: &'static str = "Tomorrow";
    pub const FUTURE: &'static str = "Future";
    pub const NO_DATE: &'static str = "No date";

    pub fn discriminant(&self) -> &'static str {
        match self {
            SectionKind::Overdue => Self::OVERDUE,
            SectionKind::Today => Self::TODAY,
            SectionKind::Tomorrow => Self::TOMORROW,
            SectionKind::Future(_) => Self::FUTURE,
            SectionKind::NoDate => Self::NO_DATE,
        }
    }

    pub fn header(&self) -> String {
        match self {
            SectionKind::Overdue => "Overdue".to_string(),
            SectionKind::Tomorrow => "Tomorrow".to_string(),
            SectionKind::Today => "Today".to_string(),
            SectionKind::Future(d) => d.format("%a, %b %-d %Y").to_string(),
            SectionKind::NoDate => "No date".to_string(),
        }
    }
}

pub struct DaySection {
    pub kind: SectionKind,
    pub todos: Vec<Todo>,
}

/// Partition of todos by date-relative section. Empty buckets are `None` / empty
/// so the view can decide what to render without filtering an `Vec` of variants.
pub struct GroupedTodos {
    pub overdue: Option<Vec<Todo>>,
    pub today: Option<Vec<Todo>>,
    pub tomorrow: Option<Vec<Todo>>,
    pub future: BTreeMap<NaiveDate, Vec<Todo>>,
    pub no_date: Option<Vec<Todo>>,
}

impl GroupedTodos {
    /// Flatten into a `Vec<DaySection>` in display order, dropping empty sections.
    /// Used by both the renderer and the section-index-based nav helpers.
    pub fn ordered_sections(&self) -> Vec<DaySection> {
        let mut out = Vec::new();
        if let Some(todos) = &self.overdue {
            out.push(DaySection {
                kind: SectionKind::Overdue,
                todos: todos.clone(),
            });
        }
        if let Some(todos) = &self.today {
            out.push(DaySection {
                kind: SectionKind::Today,
                todos: todos.clone(),
            });
        }
        if let Some(todos) = &self.tomorrow {
            out.push(DaySection {
                kind: SectionKind::Tomorrow,
                todos: todos.clone(),
            });
        }
        for (d, todos) in &self.future {
            out.push(DaySection {
                kind: SectionKind::Future(*d),
                todos: todos.clone(),
            });
        }
        if let Some(todos) = &self.no_date {
            out.push(DaySection {
                kind: SectionKind::NoDate,
                todos: todos.clone(),
            });
        }
        out
    }
}

pub fn group_todos_by_section(todos: &[Todo], today: NaiveDate) -> GroupedTodos {
    let tomorrow = today.succ_opt().expect("today + 1 day fits in NaiveDate");

    let mut overdue: Vec<Todo> = Vec::new();
    let mut today_section: Vec<Todo> = Vec::new();
    let mut tomorrow_section: Vec<Todo> = Vec::new();
    let mut future: BTreeMap<NaiveDate, Vec<Todo>> = BTreeMap::new();
    let mut no_date: Vec<Todo> = Vec::new();

    for todo in todos {
        match todo.date {
            None => no_date.push(todo.clone()),
            Some(d) if d < today => overdue.push(todo.clone()),
            Some(d) if d == today => today_section.push(todo.clone()),
            Some(d) if d == tomorrow => tomorrow_section.push(todo.clone()),
            Some(d) => future.entry(d).or_default().push(todo.clone()),
        }
    }

    GroupedTodos {
        overdue: (!overdue.is_empty()).then_some(overdue),
        today: (!today_section.is_empty()).then_some(today_section),
        tomorrow: (!tomorrow_section.is_empty()).then_some(tomorrow_section),
        future,
        no_date: (!no_date.is_empty()).then_some(no_date),
    }
}

fn find_todo_position(todo_id: Uuid, sections: &[DaySection]) -> Option<(usize, usize)> {
    for (section_idx, section) in sections.iter().enumerate() {
        if let Some(todo_idx) = section.todos.iter().position(|t| t.id == todo_id) {
            return Some((section_idx, todo_idx));
        }
    }
    None
}

fn next_todo_id(current_id: Uuid, sections: &[DaySection]) -> Option<Uuid> {
    if sections.is_empty() {
        return None;
    }
    let (section_idx, todo_idx) = find_todo_position(current_id, sections)?;

    if todo_idx + 1 < sections[section_idx].todos.len() {
        return Some(sections[section_idx].todos[todo_idx + 1].id);
    }
    if section_idx + 1 < sections.len() {
        return sections[section_idx + 1].todos.first().map(|t| t.id);
    }
    sections.first()?.todos.first().map(|t| t.id)
}

fn prev_todo_id(current_id: Uuid, sections: &[DaySection]) -> Option<Uuid> {
    if sections.is_empty() {
        return None;
    }
    let (section_idx, todo_idx) = find_todo_position(current_id, sections)?;

    if todo_idx > 0 {
        return Some(sections[section_idx].todos[todo_idx - 1].id);
    }
    if section_idx > 0 {
        return sections[section_idx - 1].todos.last().map(|t| t.id);
    }
    sections.last()?.todos.last().map(|t| t.id)
}

/// Sections built from only the todos that should currently be shown (incomplete,
/// or completed during this session). Hidden completed todos are skipped so the
/// UI never navigates onto them.
fn visible_sections(state: &TodosListState, todos: &[Todo]) -> Vec<DaySection> {
    let visible: Vec<Todo> = todos.iter().filter(|t| state.is_visible(t)).cloned().collect();
    group_todos_by_section(&visible, arc_core::time::today()).ordered_sections()
}

pub fn update(state: &mut TodosListState, todos: &mut Vec<Todo>, msg: TodosListMsg) -> Vec<Cmd> {
    match msg {
        TodosListMsg::FocusNext => {
            let sections = visible_sections(state, todos);
            if sections.is_empty() {
                return vec![Cmd::None];
            }
            if let Some(current_id) = state.selected_id {
                state.selected_id = next_todo_id(current_id, &sections);
            } else {
                state.selected_id = sections.first().and_then(|s| s.todos.first()).map(|t| t.id);
            }
            vec![Cmd::None]
        }

        TodosListMsg::FocusPrev => {
            let sections = visible_sections(state, todos);
            if sections.is_empty() {
                return vec![Cmd::None];
            }
            if let Some(current_id) = state.selected_id {
                state.selected_id = prev_todo_id(current_id, &sections);
            } else {
                state.selected_id = sections.last().and_then(|s| s.todos.last()).map(|t| t.id);
            }
            vec![Cmd::None]
        }

        TodosListMsg::ToggleComplete => {
            let Some(id) = state.selected_id else {
                return vec![Cmd::None];
            };
            let Some(todo) = todos.iter_mut().find(|t| t.id == id) else {
                return vec![Cmd::None];
            };
            todo.completed = !todo.completed;
            // Keep it on screen once completed so an accidental tick can be undone
            // without the row vanishing.
            state.visible.insert(id);

            let mut cmds = vec![Cmd::WriteTodos];
            if let crate::todos::TodoSource::NoteParsed { file, .. } = &todo.source {
                cmds.push(Cmd::SyncNoteCheckbox {
                    file: file.clone(),
                    text: todo.text.clone(),
                    complete: todo.completed,
                });
            }
            cmds
        }

        TodosListMsg::FocusSectionNext | TodosListMsg::FocusSectionPrev => {
            let sections = visible_sections(state, todos);
            if sections.is_empty() {
                return vec![Cmd::None];
            }

            let cur_section = state
                .selected_id
                .and_then(|id| find_todo_position(id, &sections))
                .map(|(s, _)| s)
                .unwrap_or(0);

            let target = if matches!(msg, TodosListMsg::FocusSectionNext) {
                (cur_section + 1) % sections.len()
            } else {
                (cur_section + sections.len() - 1) % sections.len()
            };

            state.selected_id = sections[target].todos.first().map(|t| t.id);
            vec![Cmd::None]
        }

        TodosListMsg::DeleteSelected => {
            let Some(id) = state.selected_id else {
                return vec![Cmd::None];
            };
            // Don't allow deleting note-sourced todos via the TUI.
            let is_note_parsed = todos
                .iter()
                .find(|t| t.id == id)
                .is_some_and(|t| matches!(t.source, crate::todos::TodoSource::NoteParsed { .. }));
            if is_note_parsed {
                return vec![Cmd::None];
            }

            let sections = visible_sections(state, todos);
            let next_id = next_todo_id(id, &sections);
            let prev_id = prev_todo_id(id, &sections);

            todos.retain(|t| t.id != id);

            if let Some(next) = next_id
                && todos.iter().any(|t| t.id == next)
            {
                state.selected_id = Some(next);
            } else if let Some(prev) = prev_id
                && todos.iter().any(|t| t.id == prev)
            {
                state.selected_id = Some(prev);
            } else {
                state.selected_id = todos.first().map(|t| t.id);
            }
            vec![Cmd::WriteTodos]
        }
    }
}

pub fn key_to_msg(key: KeyEvent) -> Option<TodosListMsg> {
    match key.code {
        KeyCode::Char('j') if key.modifiers == KeyModifiers::CONTROL => {
            Some(TodosListMsg::FocusSectionNext)
        }
        KeyCode::Char('k') if key.modifiers == KeyModifiers::CONTROL => {
            Some(TodosListMsg::FocusSectionPrev)
        }
        KeyCode::Char('j') | KeyCode::Down => Some(TodosListMsg::FocusNext),
        KeyCode::Char('k') | KeyCode::Up => Some(TodosListMsg::FocusPrev),
        KeyCode::Char(' ') | KeyCode::Enter => Some(TodosListMsg::ToggleComplete),
        KeyCode::Char('d') => Some(TodosListMsg::DeleteSelected),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::todos::{Todo, TodoSource};

    fn manual_dated(text: &str, date: NaiveDate) -> Todo {
        Todo {
            id: uuid::Uuid::new_v4(),
            text: text.to_string(),
            date: Some(date),
            completed: false,
            source: TodoSource::Manual,
        }
    }

    fn manual_undated(text: &str) -> Todo {
        Todo {
            id: uuid::Uuid::new_v4(),
            text: text.to_string(),
            date: None,
            completed: false,
            source: TodoSource::Manual,
        }
    }

    fn completed(text: &str) -> Todo {
        Todo {
            id: uuid::Uuid::new_v4(),
            text: text.to_string(),
            date: None,
            completed: true,
            source: TodoSource::Manual,
        }
    }

    #[test]
    fn completed_todo_loaded_at_open_is_hidden() {
        let incomplete = manual_undated("keep me");
        let done = completed("hide me");
        let todos = vec![incomplete.clone(), done.clone()];

        let mut state = TodosListState::default();
        state.refresh_visible(&todos);

        assert!(state.is_visible(&incomplete));
        assert!(!state.is_visible(&done), "pre-completed todo stays hidden");

        let sections = visible_sections(&state, &todos);
        let ids: Vec<_> = sections.iter().flat_map(|s| s.todos.iter().map(|t| t.id)).collect();
        assert_eq!(ids, vec![incomplete.id]);
    }

    #[test]
    fn toggling_complete_keeps_todo_visible() {
        let todo = manual_undated("oops");
        let mut todos = vec![todo.clone()];
        let mut state = TodosListState::default();
        state.refresh_visible(&todos);
        state.selected_id = Some(todo.id);

        update(&mut state, &mut todos, TodosListMsg::ToggleComplete);

        assert!(todos[0].completed);
        assert!(
            state.is_visible(&todos[0]),
            "a todo completed this session must remain on screen"
        );
        let sections = visible_sections(&state, &todos);
        assert_eq!(sections.iter().flat_map(|s| &s.todos).count(), 1);
    }

    #[test]
    fn refresh_visible_drops_ids_that_disappear() {
        let todo = manual_undated("gone soon");
        let mut state = TodosListState::default();
        state.refresh_visible(&[todo.clone()]);
        assert!(state.visible.contains(&todo.id));

        // Todo no longer in the list (e.g. note line deleted) -> id pruned.
        state.refresh_visible(&[]);
        assert!(state.visible.is_empty());
    }

    #[test]
    fn groups_overdue_today_tomorrow_future_nodate_in_order() {
        let today = NaiveDate::from_ymd_opt(2026, 5, 31).unwrap();
        let tomorrow = today.succ_opt().unwrap();
        let yesterday = today.pred_opt().unwrap();
        let next_week = today + chrono::Duration::days(7);

        let todos = vec![
            manual_dated("future", next_week),
            manual_dated("yesterday", yesterday),
            manual_undated("no date"),
            manual_dated("today", today),
            manual_dated("tomorrow", tomorrow),
        ];

        let grouped = group_todos_by_section(&todos, today);
        assert!(grouped.overdue.is_some());
        assert!(grouped.today.is_some());
        assert!(grouped.tomorrow.is_some());
        assert_eq!(grouped.future.len(), 1);
        assert!(grouped.no_date.is_some());

        let sections = grouped.ordered_sections();
        let kinds: Vec<_> = sections.iter().map(|s| s.kind.discriminant()).collect();
        assert_eq!(
            kinds,
            vec![
                SectionKind::OVERDUE,
                SectionKind::TODAY,
                SectionKind::TOMORROW,
                SectionKind::FUTURE,
                SectionKind::NO_DATE,
            ]
        );
    }
}
