use throbber_widgets_tui::ThrobberState;
use tokio::task::JoinHandle;
use tui_scrollview::ScrollViewState;

use std::path::PathBuf;

use crate::{
    api::{epic::EpicSlim, iteration::Iteration, story::Story},
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
    Stories, // Current default: story list
    Epics,      // Future: browse all epics
    Notes,      // Future: browse notes directory
    Search,     // Future: search across stories/notes
    Iterations, // browse iterations
}

impl ViewType {
    // NOTE: keep the ordering of the below three items consistent with eachother
    pub const ALL: &[Self] = &[
        ViewType::Stories,
        ViewType::Iterations,
        ViewType::Notes,
        ViewType::Epics,
        ViewType::Search,
    ];

    pub fn next(self) -> Self {
        match self {
            ViewType::Stories => ViewType::Iterations,
            ViewType::Iterations => ViewType::Notes,
            ViewType::Notes => ViewType::Epics,
            ViewType::Epics => ViewType::Search,
            ViewType::Search => ViewType::Stories,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            ViewType::Stories => ViewType::Search,
            ViewType::Search => ViewType::Epics,
            ViewType::Epics => ViewType::Notes,
            ViewType::Notes => ViewType::Iterations,
            ViewType::Iterations => ViewType::Stories,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            ViewType::Stories => "Stories",
            ViewType::Epics => "Epics",
            ViewType::Notes => "Notes",
            ViewType::Search => "Search",
            ViewType::Iterations => "Iterations",
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
    pub iterations: Vec<Iteration>,
    pub epics: Vec<EpicSlim>,
    pub current_iterations: Option<Vec<Iteration>>,
    pub active_story: Option<Story>,
    pub async_handles: Vec<JoinHandle<()>>,
}

#[derive(Default, Debug)]
pub struct DescriptionModalState {
    pub is_showing: bool,
    pub scroll_view_state: ScrollViewState,
    pub story: Option<Story>,
}

#[derive(Default, Debug)]
pub struct UiState {
    pub active_view: ViewType,
    pub story_list: StoryListState,
    pub notes_list: NotesListState,
    pub iteration_list: IterationListState,
    pub epic_list: EpicListState,
    pub action_menu: ActionMenuState,
    pub description_modal: DescriptionModalState,
    pub show_keybinds_panel: bool,
    pub errors: Vec<ErrorInfo>,
    pub loading: LoadingState,
    pub throbber_state: ThrobberState,
}

#[derive(Clone, Debug, Default)]
pub struct IterationListState {
    pub selected_id: Option<i32>,
    pub search_query: String,
    pub search_active: bool,
}

#[derive(Clone, Debug, Default)]
pub struct EpicListState {
    pub selected_id: Option<i32>,
    pub search_query: String,
    pub search_active: bool,
}

#[derive(Clone, Debug)]
pub struct StoryListState {
    pub selected_story_id: Option<i32>,
    pub show_finished: bool,
}

impl Default for StoryListState {
    fn default() -> Self {
        Self {
            selected_story_id: Default::default(),
            show_finished: true,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct NotesListState {
    pub selected_path: Option<PathBuf>,
    pub daily_notes: Vec<PathBuf>,
    pub other_notes: Vec<PathBuf>,
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
                epics: cache.epics.clone(),
                current_iterations: cache.current_iterations.clone(),
                active_story: cache.active_story.clone(),
                async_handles: Vec::new(),
                iterations: cache.iterations.clone(),
            },
            ui: UiState::default(),
            config,
            cache,
        };
        model.ui.story_list.selected_story_id = model.data.stories.first().map(|s| s.id);
        model.ui.epic_list.selected_id = model.data.epics.first().map(|e| e.id);
        model
    }
}
