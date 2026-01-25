use crossterm::event::KeyEvent;

use crate::api::{epic::Epic, iteration::Iteration, story::Story};

#[derive(Debug, Clone)]
pub enum Msg {
    KeyPressed(KeyEvent),
    StoryList(StoryListMsg),
    StoriesLoaded {
        stories: Vec<Story>,
        from_cache: bool,
    },
    EpicsLoaded(Vec<Epic>),
    IterationLoaded(Iteration),
    NoteOpened,
    CacheWritten,
    Error(String),
    Quit,
}

#[derive(Debug, Clone, Copy)]
pub enum StoryListMsg {
    SelectNext,
    SelectPrev,
    ToggleExpand,
    OpenNote,
}
