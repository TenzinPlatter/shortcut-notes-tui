use throbber_widgets_tui::ThrobberState;

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
            LoadingState::FetchingIteration => "Fetching iterations...",
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

#[derive(Debug)]
pub struct Model {
    pub data: DataState,
    pub ui: UiState,
    pub config: Config,
    pub cache: Cache,
}

#[derive(Default, Debug)]
pub struct DataState {
    pub stories: Vec<Story>,
    pub epics: Vec<Epic>,
    pub current_iterations: Option<Vec<Iteration>>,
    pub active_story: Option<Story>,
}

#[derive(Clone, Default, Debug)]
pub struct DescriptionModalState {
    pub is_showing: bool,
    pub scroll_offset: u16,
    pub story: Option<Story>,
}

#[derive(Default, Debug)]
pub struct UiState {
    pub active_view: ViewType,
    pub story_list: StoryListState,
    pub action_menu: ActionMenuState,
    pub description_modal: DescriptionModalState,
    pub errors: Vec<ErrorInfo>,
    pub loading: LoadingState,
    pub throbber_state: ThrobberState,
}

#[derive(Clone, Debug, Default)]
pub struct StoryListState {
    pub selected_story_id: Option<i32>,
}

impl StoryListState {
    /// Returns the index of the selected story in the given slice, if it exists.
    pub fn selected_index(&self, stories: &[Story]) -> Option<usize> {
        let id = self.selected_story_id?;
        stories.iter().position(|s| s.id == id)
    }
}

impl DataState {
    pub fn current_iterations_ref(&self) -> Option<Vec<&Iteration>> {
        self.current_iterations.as_ref().map(|v| v.iter().collect())
    }
}

impl Model {
    pub fn from_cache_and_config(cache: Cache, config: Config) -> Model {
        let mut model = Model {
            data: DataState {
                stories: cache.iteration_stories.clone().unwrap_or_default(),
                epics: vec![],
                current_iterations: cache.current_iterations.clone(),
                active_story: cache.active_story.clone(),
            },
            ui: UiState::default(),
            config,
            cache,
        };
        model.ui.story_list.selected_story_id = model.data.stories.first().map(|s| s.id);
        model
    }
}
