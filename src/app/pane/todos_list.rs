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

pub fn group_todos_by_section(todos: &[Todo], today: NaiveDate) -> Vec<DaySection> {
    let tomorrow = today.succ_opt().expect("today + 1 day fits in NaiveDate");

    let mut overdue: Vec<Todo> = Vec::new();
    let mut today_section: Vec<Todo> = Vec::new();
    let mut tomorrow_section: Vec<Todo> = Vec::new();
    let mut future: std::collections::BTreeMap<NaiveDate, Vec<Todo>> = Default::default();
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

    let mut out = Vec::new();
    if !overdue.is_empty() {
        out.push(DaySection { kind: SectionKind::Overdue, todos: overdue });
    }
    if !today_section.is_empty() {
        out.push(DaySection { kind: SectionKind::Today, todos: today_section });
    }
    if !tomorrow_section.is_empty() {
        out.push(DaySection { kind: SectionKind::Tomorrow, todos: tomorrow_section });
    }
    for (d, todos) in future {
        out.push(DaySection { kind: SectionKind::Future(d), todos });
    }
    if !no_date.is_empty() {
        out.push(DaySection { kind: SectionKind::NoDate, todos: no_date });
    }

    out
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

pub fn update(
    state: &mut TodosListState,
    todos: &mut Vec<Todo>,
    msg: TodosListMsg,
) -> Vec<Cmd> {
    match msg {
        TodosListMsg::FocusNext => {
            if todos.is_empty() {
                return vec![Cmd::None];
            }
            let sections = group_todos_by_section(todos, crate::time::today());
            if let Some(current_id) = state.selected_id {
                state.selected_id = next_todo_id(current_id, &sections);
            } else {
                state.selected_id = sections.first().and_then(|s| s.todos.first()).map(|t| t.id);
            }
            vec![Cmd::None]
        }

        TodosListMsg::FocusPrev => {
            if todos.is_empty() {
                return vec![Cmd::None];
            }
            let sections = group_todos_by_section(todos, crate::time::today());
            if let Some(current_id) = state.selected_id {
                state.selected_id = prev_todo_id(current_id, &sections);
            } else {
                state.selected_id = sections.last().and_then(|s| s.todos.last()).map(|t| t.id);
            }
            vec![Cmd::None]
        }

        TodosListMsg::ToggleComplete => {
            if let Some(id) = state.selected_id {
                if let Some(todo) = todos.iter_mut().find(|t| t.id == id) {
                    todo.completed = !todo.completed;
                }
            }
            vec![Cmd::WriteTodos]
        }

        TodosListMsg::FocusSectionNext | TodosListMsg::FocusSectionPrev => {
            let sections = group_todos_by_section(todos, crate::time::today());
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
            if let Some(id) = state.selected_id {
                let sections = group_todos_by_section(todos, crate::time::today());
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

        let sections = group_todos_by_section(&todos, today);
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
