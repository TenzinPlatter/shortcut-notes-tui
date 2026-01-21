use std::fmt::Display;

use crossterm::event::{KeyCode, KeyEvent};

pub trait KeyHandler {
    /// Handle a key event. Returns whether the event was consumed by the handler, i.e. whether
    /// further processing should be stopped.
    fn handle_key_event(&mut self, _key_event: KeyEvent) -> bool {
        false
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum AppKey {
    Left,
    Right,
    Up,
    Down,
    Quit,
    ShowErrorDetails,
    Select,
}

impl AppKey {
    pub fn as_keycode(&self) -> KeyCode {
        // NOTE: also add to function below when adding a variant
        match self {
            AppKey::Left => KeyCode::Char('h'),
            AppKey::Right => KeyCode::Char('l'),
            AppKey::Up => KeyCode::Char('k'),
            AppKey::Down => KeyCode::Char('j'),
            AppKey::Quit => KeyCode::Char('q'),
            AppKey::ShowErrorDetails => KeyCode::Char('d'),
            AppKey::Select => KeyCode::Enter,
        }
    }
}

impl TryFrom<KeyCode> for AppKey {
    type Error = ();

    fn try_from(key_code: KeyCode) -> Result<Self, Self::Error> {
        match key_code {
            KeyCode::Char('h') => Ok(AppKey::Left),
            KeyCode::Char('l') => Ok(AppKey::Right),
            KeyCode::Char('k') => Ok(AppKey::Up),
            KeyCode::Char('j') => Ok(AppKey::Down),
            KeyCode::Char('q') => Ok(AppKey::Quit),
            KeyCode::Char('d') => Ok(AppKey::ShowErrorDetails),
            KeyCode::Enter => Ok(AppKey::Select),
            _ => Err(()),
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
