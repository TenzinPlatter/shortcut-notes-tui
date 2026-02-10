use crossterm::event::{KeyCode, KeyEvent};

use crate::{
    app::{
        App,
        cmd::Cmd,
        model::LoadingState,
        msg::Msg,
        pane::{action_menu, description_modal, story_list},
    },
    dbg_file,
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
                if self.model.ui.story_list.selected_index.is_none() && !stories.is_empty() {
                    self.model.ui.story_list.selected_index = Some(0);
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

                let mut res = vec![Cmd::WriteCache];
                for iteration in iterations.iter() {
                    res.push(Cmd::FetchStories {
                        iteration_id: iteration.id,
                    });
                }

                res
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
                if let Some(idx) = self.model.ui.story_list.selected_index {
                    let hovered_story = &self.model.data.stories[idx];
                    action_menu::update(
                        &mut self.model.ui,
                        &self.model.data,
                        menu_msg,
                        hovered_story,
                    )
                } else {
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
                let (visible_height, total_lines) =
                    if let Some(story) = &self.model.ui.description_modal.story {
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

        match key.code {
            KeyCode::Char('q') => return self.update(Msg::Quit),

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
                if let Some(idx) = self.model.ui.story_list.selected_index
                    && let Some(story) = self.model.data.stories.get(idx)
                {
                    description_modal::open(
                        &mut self.model.ui.description_modal,
                        story.clone(),
                    );
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
