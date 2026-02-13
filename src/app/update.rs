use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::{
    app::{
        App,
        cmd::Cmd,
        model::LoadingState,
        msg::Msg,
        pane::{action_menu, description_modal, story_list},
    },
    dbg_file,
    error::ErrorInfo,
    view::description_modal::DescriptionModal,
};

impl App {
    pub fn update(&mut self, msg: Msg) -> Vec<Cmd> {
        dbg_file!("Update: {:?}", msg);

        match msg {
            Msg::Quit => {
                self.exit = true;
                vec![Cmd::None]
            }

            Msg::KeyPressed(key_event) => self.handle_key_input(key_event),

            Msg::StoryList(story_msg) => story_list::update(
                &mut self.model.ui.story_list,
                &self.model.data.stories,
                self.model.data.current_iterations_ref(),
                story_msg,
            ),

            Msg::StoriesLoaded {
                stories,
                from_cache,
            } => {
                // Only transition to Loaded on fresh API data, not cached
                // This keeps spinner showing during background refresh
                if !from_cache {
                    self.model.ui.loading = LoadingState::Loaded;
                }

                // Select first story if none selected and list is non-empty
                if self.model.ui.story_list.selected_story_id.is_none() && !stories.is_empty() {
                    self.model.ui.story_list.selected_story_id = stories.first().map(|s| s.id);
                }

                if !from_cache
                    && self.model.data.stories.len() == stories.len()
                    && self
                        .model
                        .data
                        .stories
                        .iter()
                        .zip(stories.iter())
                        .all(|(a, b)| a.id == b.id)
                {
                    return vec![Cmd::None];
                }

                self.model.data.stories = stories.clone();

                // Reconcile selection: if selected story no longer exists, select first
                if let Some(selected_id) = self.model.ui.story_list.selected_story_id
                    && !stories.iter().any(|s| s.id == selected_id)
                {
                    self.model.ui.story_list.selected_story_id = stories.first().map(|s| s.id);
                }

                // Reconcile description modal
                if self.model.ui.description_modal.is_showing
                    && let Some(ref modal_story) = self.model.ui.description_modal.story
                {
                    match stories.iter().find(|s| s.id == modal_story.id) {
                        Some(fresh_story) => {
                            // Update modal with fresh data
                            self.model.ui.description_modal.story = Some(fresh_story.clone());
                        }
                        None => {
                            // Story gone — close modal, show error
                            self.model.ui.description_modal.is_showing = false;
                            self.model.ui.description_modal.story = None;
                            self.model.ui.errors.push(ErrorInfo::new(
                                "Story no longer available".to_string(),
                                "The story was removed or moved out of this iteration".to_string(),
                            ));
                        }
                    }
                }

                // Reconcile action menu
                if self.model.ui.action_menu.is_showing
                    && let Some(target_id) = self.model.ui.action_menu.target_story_id
                    && !stories.iter().any(|s| s.id == target_id)
                {
                    // Story gone — close menu, show error
                    self.model.ui.action_menu.is_showing = false;
                    self.model.ui.action_menu.target_story_id = None;
                    self.model.ui.errors.push(ErrorInfo::new(
                        "Story no longer available".to_string(),
                        "The story was removed or moved out of this iteration".to_string(),
                    ));
                }

                // Reconcile active story
                if let Some(ref active) = self.model.data.active_story
                    && !stories.iter().any(|s| s.id == active.id)
                {
                    // Active story no longer in iteration — clear it
                    self.model.data.active_story = None;
                    self.model.cache.active_story = None;
                    self.model.ui.errors.push(ErrorInfo::new(
                        "Active story cleared".to_string(),
                        "The active story is no longer in the current iteration".to_string(),
                    ));
                }

                self.model.cache.iteration_stories = Some(stories);

                vec![Cmd::WriteCache]
            }

            Msg::EpicsLoaded(epics) => {
                self.model.data.epics = epics;
                vec![Cmd::None]
            }

            Msg::IterationsLoaded(iterations) => {
                self.model.data.current_iterations = Some(iterations.clone());
                self.model.cache.current_iterations = Some(iterations.clone());
                self.model.ui.loading = LoadingState::FetchingStories;

                dbg_file!(
                    "Got iterations: {:?}",
                    iterations.iter().map(|it| it.id).collect::<Vec<_>>()
                );
                vec![
                    Cmd::WriteCache,
                    Cmd::FetchStories {
                        iteration_ids: iterations.iter().map(|it| it.id).collect(),
                    },
                ]
            }

            Msg::SwitchToView(view_type) => {
                self.model.ui.active_view = view_type;
                vec![Cmd::None]
            }

            Msg::NoteOpened => {
                vec![Cmd::None]
            }

            Msg::CacheWritten => vec![Cmd::None],

            Msg::Error(e) => {
                self.model.ui.errors.push(e);
                // Stop loading spinner on error
                self.model.ui.loading = LoadingState::Loaded;
                vec![Cmd::None]
            }

            Msg::ActionMenu(menu_msg) => {
                let story = self
                    .model
                    .ui
                    .action_menu
                    .target_story_id
                    .and_then(|id| self.model.data.stories.iter().find(|s| s.id == id));

                if let Some(hovered_story) = story {
                    action_menu::update(
                        &mut self.model.ui,
                        &self.model.data,
                        menu_msg,
                        hovered_story,
                    )
                } else {
                    // Target story no longer exists, close menu
                    self.model.ui.action_menu.is_showing = false;
                    vec![Cmd::None]
                }
            }

            Msg::ToggleActionMenu => {
                vec![Cmd::ActionMenuVisibility(
                    !self.model.ui.action_menu.is_showing,
                )]
            }

            Msg::DescriptionModal(modal_msg) => {
                // Calculate dimensions for scroll bounds
                // Use a reasonable default if we can't get terminal size
                let (visible_height, total_lines) = if let Some(story) =
                    &self.model.ui.description_modal.story
                {
                    // Approximate based on typical terminal size
                    // These will be recalculated properly during render
                    let approx_height = 20u16;
                    let approx_width = 60u16;
                    let total =
                        DescriptionModal::calculate_total_lines(&story.description, approx_width);
                    (approx_height, total)
                } else {
                    (20, 0)
                };

                description_modal::update(
                    &mut self.model.ui.description_modal,
                    modal_msg,
                    visible_height,
                    total_lines,
                )
            }
        }
    }

    fn handle_key_input(&mut self, key: KeyEvent) -> Vec<Cmd> {
        // Description modal takes priority (rendered on top of everything)
        if self.model.ui.description_modal.is_showing {
            return if let Some(msg) = description_modal::key_to_msg(key) {
                self.update(Msg::DescriptionModal(msg))
            } else {
                vec![Cmd::None]
            };
        }

        // action menu is rendered as an overlay, so swallows all keybinds when showing
        if self.model.ui.action_menu.is_showing {
            return if let Some(msg) = action_menu::key_to_msg(key) {
                self.update(Msg::ActionMenu(msg))
            } else {
                vec![Cmd::None]
            };
        }

        if should_quit(&key) {
            return self.update(Msg::Quit);
        }

        match key.code {
            // View switching (Tab/Shift+Tab)
            KeyCode::Tab => {
                let next_view = self.model.ui.active_view.next();
                return self.update(Msg::SwitchToView(next_view));
            }

            KeyCode::BackTab => {
                let prev_view = self.model.ui.active_view.prev();
                return self.update(Msg::SwitchToView(prev_view));
            }

            KeyCode::Enter => return self.update(Msg::ToggleActionMenu),

            // Open description modal with Space
            KeyCode::Char(' ') => {
                let story = self
                    .model
                    .ui
                    .story_list
                    .selected_story_id
                    .and_then(|id| self.model.data.stories.iter().find(|s| s.id == id));

                if let Some(story) = story {
                    description_modal::open(&mut self.model.ui.description_modal, story.clone());
                }
                return vec![Cmd::None];
            }

            _ => {}
        }

        // Route to active view's key handler
        if let Some(msg) = story_list::key_to_msg(key) {
            return self.update(Msg::StoryList(msg));
        }

        vec![Cmd::None]
    }
}

fn should_quit(key: &KeyEvent) -> bool {
    matches!(key.code, KeyCode::Char('q'))
        || (matches!(key.code, KeyCode::Char('c')) && key.modifiers.contains(KeyModifiers::CONTROL))
}
