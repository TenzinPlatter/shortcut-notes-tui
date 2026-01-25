use crossterm::event::KeyEvent;

use crate::{
    app::{App, cmd::Cmd, msg::Msg, pane::story_list},
    dbg_file,
    keys::AppKey,
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

            Msg::NoteOpened => vec![Cmd::None],
            Msg::CacheWritten => vec![Cmd::None],

            Msg::Error(e) => {
                eprintln!("Error: {:?}", e);
                vec![Cmd::None]
            }
        }
    }

    fn handle_key_input(&mut self, key: KeyEvent) -> Vec<Cmd> {
        match key.code.try_into() {
            Ok(AppKey::Quit) => return self.update(Msg::Quit),

            // View switching (Tab/Shift+Tab)
            Ok(AppKey::Tab) => {
                let next_view = self.model.ui.active_view.next();
                return self.update(Msg::SwitchToView(next_view));
            }
            Ok(AppKey::BackTab) => {
                let prev_view = self.model.ui.active_view.prev();
                return self.update(Msg::SwitchToView(prev_view));
            }

            _ => {}
        }

        // Route to active view's key handler
        // For now, only story_list handles keys
        if let Some(msg) = story_list::key_to_msg(key) {
            self.update(Msg::StoryList(msg))
        } else {
            vec![Cmd::None]
        }
    }
}
