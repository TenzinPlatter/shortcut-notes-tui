use std::path::PathBuf;

use crossterm::event::{KeyCode, KeyEvent};
use slugify::slugify;

use crate::{
    app::{cmd::Cmd, model::CreateNoteModalState, msg::CreateNoteModalMsg},
    config::Config,
};

pub fn update(
    state: &mut CreateNoteModalState,
    config: &Config,
    msg: CreateNoteModalMsg,
) -> Vec<Cmd> {
    match msg {
        CreateNoteModalMsg::Open => {
            state.is_showing = true;
            state.input.clear();
            vec![Cmd::None]
        }

        CreateNoteModalMsg::Close => {
            state.is_showing = false;
            vec![Cmd::None]
        }

        CreateNoteModalMsg::TextInput(c) => {
            state.input.push(c);
            vec![Cmd::None]
        }

        CreateNoteModalMsg::TextBackspace => {
            state.input.pop();
            vec![Cmd::None]
        }

        CreateNoteModalMsg::Accept => {
            if state.input.is_empty() {
                return vec![Cmd::None];
            }
            let name = state.input.clone();
            let slug = slugify!(&name);
            let path: PathBuf = config
                .notes_dir
                .join("scratch")
                .join(format!("{}.md", slug));
            state.is_showing = false;
            vec![Cmd::OpenScratchNote { path, name }]
        }
    }
}

pub fn key_to_msg(key: KeyEvent) -> Option<CreateNoteModalMsg> {
    match key.code {
        KeyCode::Esc => Some(CreateNoteModalMsg::Close),
        KeyCode::Enter => Some(CreateNoteModalMsg::Accept),
        KeyCode::Backspace => Some(CreateNoteModalMsg::TextBackspace),
        KeyCode::Char(c) => Some(CreateNoteModalMsg::TextInput(c)),
        _ => None,
    }
}
