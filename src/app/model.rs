use std::collections::HashSet;

use crate::{
    api::{epic::Epic, iteration::Iteration, story::Story},
    app::pane::action_menu::ActionMenuState,
    cache::Cache,
    config::Config,
    error::ErrorInfo,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LoadingState {
    #[default]
    FetchingIteration,
    FetchingStories,
    Loaded,
}

impl LoadingState {
    pub fn label(&self) -> &'static str {
        match self {
            LoadingState::FetchingIteration => "Fetching iteration...",
            LoadingState::FetchingStories => "Loading stories...",
            LoadingState::Loaded => "",
        }
    }

    pub fn is_loading(&self) -> bool {
        !matches!(self, LoadingState::Loaded)
    }
}

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
    pub active_story: Option<Story>,
}

#[derive(Default)]
pub struct UiState {
    pub active_view: ViewType,
    pub story_list: StoryListState,
    pub action_menu: ActionMenuState,
    pub errors: Vec<ErrorInfo>,
    pub loading: LoadingState,
    pub throbber_state: throbber_widgets_tui::ThrobberState,
}


#[derive(Clone, Default)]
pub struct StoryListState {
    pub selected_index: Option<usize>,
    pub expanded_items: HashSet<usize>,
}
