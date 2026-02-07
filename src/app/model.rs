use std::collections::HashSet;

use ratatui::widgets::ListState;

use crate::{
    api::{epic::Epic, iteration::Iteration, story::Story}, app::pane::action_menu::{self, ActionMenuState}, cache::Cache, config::Config, error::ErrorInfo
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ViewType {
    #[default]
    Iteration, // Current default: story list
    Epics,  // Future: browse all epics
    Notes,  // Future: browse notes directory
    Search, // Future: search across stories/notes
}

impl ViewType {
    pub fn next(self) -> Self {
        match self {
            ViewType::Iteration => ViewType::Epics,
            ViewType::Epics => ViewType::Notes,
            ViewType::Notes => ViewType::Search,
            ViewType::Search => ViewType::Iteration,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            ViewType::Iteration => ViewType::Search,
            ViewType::Search => ViewType::Notes,
            ViewType::Notes => ViewType::Epics,
            ViewType::Epics => ViewType::Iteration,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            ViewType::Iteration => "Iteration",
            ViewType::Epics => "Epics",
            ViewType::Notes => "Notes",
            ViewType::Search => "Search",
        }
    }
}

pub struct Model {
    pub data: DataState,
    pub ui: UiState,
    pub config: Config,
    pub cache: Cache,
}

#[derive(Default)]
pub struct DataState {
    pub stories: Vec<Story>,
    pub epics: Vec<Epic>,
    pub current_iteration: Option<Iteration>,
}

pub struct UiState {
    pub active_view: ViewType,
    pub story_list: StoryListState,
    pub action_menu: ActionMenuState,
    pub errors: Vec<ErrorInfo>,
}

impl UiState {
    pub fn new(active_story: Option<Story>) -> UiState {
        Self {
            active_view: ViewType::default(),
            action_menu: ActionMenuState::default(),
            errors: vec![ErrorInfo::new("hi there", "hello!")],
            story_list: StoryListState {
                selected_index: Some(0),
                expanded_items: HashSet::default(),
                active_story,
            },
        }
    }
}

#[derive(Clone)]
pub struct StoryListState {
    pub selected_index: Option<usize>,
    pub expanded_items: HashSet<usize>,
    // TODO: reference
    pub active_story: Option<Story>,
}

impl Default for StoryListState {
    fn default() -> Self {
        Self {
            selected_index: Some(0),
            expanded_items: HashSet::default(),
            active_story: None,
        }
    }
}

