use crossterm::event::KeyEvent;

use crate::api::{epic::Epic, iteration::Iteration, story::Story};
use crate::app::model::ViewType;
use crate::error::ErrorInfo;

#[derive(Debug, Clone)]
pub enum Msg {
    KeyPressed(KeyEvent),
    StoryList(StoryListMsg),
    NotesList(NotesListMsg),
    ActionMenu(ActionMenuMsg),
    DescriptionModal(DescriptionModalMsg),
    StoriesLoaded {
        stories: Vec<Story>,
        from_cache: bool,
    },
    EpicsLoaded(Vec<Epic>),
    IterationsLoaded(Vec<Iteration>),
    SwitchToView(ViewType),
    NoteOpened,
    ToggleActionMenu,
    ToggleKeybindsPanel,
    CacheWritten,
    Error(ErrorInfo),
    Quit,
}

#[derive(Debug, Clone, Copy)]
pub enum StoryListMsg {
    FocusNext,
    FocusPrev,
    OpenNote,
    SelectStory,
    TmuxEnter,
    EditStoryContents,
    OpenInBrowser,
    ToggleFinished,
}

#[derive(Debug, Clone, Copy)]
pub enum ActionMenuMsg {
    FocusNext,
    FocusPrev,
    Accept,
    Close,
}

#[derive(Debug, Clone, Copy)]
pub enum NotesListMsg {
    FocusNext,
    FocusPrev,
    OpenNote,
}

#[derive(Debug, Clone)]
pub enum DescriptionModalMsg {
    Open,
    Close,
    ScrollUp,
    ScrollDown,
    ScrollPageUp,
    ScrollPageDown,
    ScrollToTop,
    ScrollToBottom,
}
