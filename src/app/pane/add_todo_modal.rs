use crossterm::event::{KeyCode, KeyEvent};

use crate::{
    app::{cmd::Cmd, model::AddTodoModalState, msg::AddTodoModalMsg},
    todos::Todo,
};

pub fn update(
    state: &mut AddTodoModalState,
    todos: &mut Vec<Todo>,
    msg: AddTodoModalMsg,
) -> Vec<Cmd> {
    match msg {
        AddTodoModalMsg::Open => {
            state.is_showing = true;
            state.input.clear();
            vec![Cmd::None]
        }

        AddTodoModalMsg::Close => {
            state.is_showing = false;
            vec![Cmd::None]
        }

        AddTodoModalMsg::TextInput(c) => {
            state.input.push(c);
            vec![Cmd::None]
        }

        AddTodoModalMsg::TextBackspace => {
            state.input.pop();
            vec![Cmd::None]
        }

        AddTodoModalMsg::Accept => {
            if state.input.is_empty() {
                return vec![Cmd::None];
            }
            let text = state.input.clone();
            let today = crate::time::today();
            todos.push(Todo::new_manual(text, today));
            state.is_showing = false;
            state.input.clear();
            vec![Cmd::WriteTodos]
        }
    }
}

pub fn key_to_msg(key: KeyEvent) -> Option<AddTodoModalMsg> {
    match key.code {
        KeyCode::Esc => Some(AddTodoModalMsg::Close),
        KeyCode::Enter => Some(AddTodoModalMsg::Accept),
        KeyCode::Backspace => Some(AddTodoModalMsg::TextBackspace),
        KeyCode::Char(c) => Some(AddTodoModalMsg::TextInput(c)),
        _ => None,
    }
}
