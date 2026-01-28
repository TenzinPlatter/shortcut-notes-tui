use crossterm::event::KeyEvent;

use crate::api::{epic::Epic, iteration::Iteration, story::Story};
use crate::app::model::ViewType;

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
    SwitchToView(ViewType),
    NoteOpened(Story),
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
