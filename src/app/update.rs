use crossterm::event::{KeyCode, KeyEvent};

use crate::{
    app::{
        App,
        cmd::Cmd,
        msg::Msg,
        pane::{action_menu, story_list},
    },
    dbg_file,
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
                self.model.data.current_iteration.as_ref(),
                story_msg,
            ),

            Msg::StoriesLoaded {
                stories,
                from_cache,
            } => {
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

            Msg::IterationLoaded(iteration) => {
                self.model.data.current_iteration = Some(iteration.clone());
                self.model.cache.current_iteration = Some(iteration.clone());

                vec![Cmd::WriteCache, Cmd::FetchStories { iteration }]
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
                vec![Cmd::None]
            }

            Msg::ActionMenu(menu_msg) => {
                if let Some(idx) = self.model.ui.story_list.selected_index {
                    let hovered_story = &self.model.data.stories[idx];
                    action_menu::update(&mut self.model.ui, &self.model.data, menu_msg, hovered_story)
                } else {
                    vec![Cmd::None]
                }
            }

            Msg::ToggleActionMenu => {
                vec![Cmd::ActionMenuVisibility(
                    !self.model.ui.action_menu.is_showing,
                )]
            }
        }
    }

    fn handle_key_input(&mut self, key: KeyEvent) -> Vec<Cmd> {
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

            _ => {}
        }

        // Route to active view's key handler
        if let Some(msg) = story_list::key_to_msg(key) {
            return self.update(Msg::StoryList(msg));
        }

        vec![Cmd::None]
    }
}
