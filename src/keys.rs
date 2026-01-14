use std::fmt::Display;

use crossterm::event::{KeyCode, KeyEvent};

pub trait KeyHandler {
    fn handle_key_event(&mut self, key_event: KeyEvent);
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum AppKey {
    Left,
    Right,
    Up,
    Down,
    Quit,
}

impl AppKey {
    pub fn as_keycode(&self) -> KeyCode {
        match self {
            AppKey::Left => KeyCode::Char('h'),
            AppKey::Right => KeyCode::Char('l'),
            AppKey::Up => KeyCode::Char('k'),
            AppKey::Down => KeyCode::Char('j'),
            AppKey::Quit => KeyCode::Char('q'),
        }
    }
}

// TODO: make this try_from
impl From<KeyCode> for AppKey {
    fn from(key_code: KeyCode) -> Self {
        match key_code {
            KeyCode::Char('h') => AppKey::Left,
            KeyCode::Char('l') => AppKey::Right,
            KeyCode::Char('k') => AppKey::Up,
            KeyCode::Char('j') => AppKey::Down,
            KeyCode::Char('q') => AppKey::Quit,
            _ => panic!("Unsupported key code for AppKey"),
        }
    }
}

impl From<AppKey> for KeyCode {
    fn from(app_key: AppKey) -> Self {
        app_key.as_keycode()
    }
}

impl From<&AppKey> for KeyCode {
    fn from(app_key: &AppKey) -> Self {
        app_key.as_keycode()
    }
}

impl From<&AppKey> for String {
    fn from(app_key: &AppKey) -> Self {
        app_key.as_keycode().to_string()
    }
}

impl From<AppKey> for String {
    fn from(app_key: AppKey) -> Self {
        app_key.as_keycode().to_string()
    }
}

impl Display for AppKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_keycode())
    }
}
