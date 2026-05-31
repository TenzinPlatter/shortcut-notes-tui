use std::collections::BTreeMap;

use chrono::NaiveDate;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use uuid::Uuid;

use crate::{
    app::{cmd::Cmd, msg::TodosListMsg},
    todos::Todo,
};

pub use crate::app::model::TodosListState;

pub struct DaySection {
    pub date: NaiveDate,
    pub todos: Vec<Todo>,
}

pub fn group_todos_by_date(todos: &[Todo]) -> Vec<DaySection> {
    let mut map: BTreeMap<NaiveDate, Vec<Todo>> = BTreeMap::new();
    for todo in todos {
        if let Some(date) = todo.date {
            map.entry(date).or_default().push(todo.clone());
        }
    }
    map.into_iter()
        .rev()
        .map(|(date, todos)| DaySection { date, todos })
        .collect()
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
            let sections = group_todos_by_date(todos);
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
            let sections = group_todos_by_date(todos);
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
            let sections = group_todos_by_date(todos);
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
                let sections = group_todos_by_date(todos);
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
