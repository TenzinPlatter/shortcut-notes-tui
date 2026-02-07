use std::fmt::Display;

use crossterm::event::KeyCode;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum AppKey {
    Left,
    Right,
    Up,
    Down,
    Quit,
    ShowErrorDetails,
    Select,
    Edit,
    Tab,
    BackTab,
    SetActive,
    TmuxEnter,
    EditNoteContents,
    ToggleActionMenu,
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
            AppKey::Select => KeyCode::Char(' '),
            AppKey::Edit => KeyCode::Char('p'), // TODO: put this back to something else? used to
            // be enter
            AppKey::Tab => KeyCode::Tab,
            AppKey::BackTab => KeyCode::BackTab,
            AppKey::SetActive => KeyCode::Char('a'),
            AppKey::TmuxEnter => KeyCode::Char('t'),
            AppKey::EditNoteContents => KeyCode::Char('e'),
            AppKey::ToggleActionMenu => KeyCode::Enter,
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
            KeyCode::Char(' ') => Ok(AppKey::Select),
            KeyCode::Char('p') => Ok(AppKey::Edit),
            KeyCode::Tab => Ok(AppKey::Tab),
            KeyCode::BackTab => Ok(AppKey::BackTab),
            KeyCode::Char('a') => Ok(AppKey::SetActive),
            KeyCode::Char('t') => Ok(AppKey::TmuxEnter),
            KeyCode::Char('e') => Ok(AppKey::EditNoteContents),
            KeyCode::Enter => Ok(AppKey::ToggleActionMenu),
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
