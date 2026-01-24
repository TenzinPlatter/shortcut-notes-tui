use crossterm::event::KeyEvent;

use crate::{
    app::{
        cmd::Cmd,
        model::PaneId,
        msg::Msg,
        pane::story_list,
        App,
    },
    keys::AppKey,
};

impl App {
    pub fn update(&mut self, msg: Msg) -> Vec<Cmd> {
        match msg {
            Msg::Quit => {
                self.exit = true;
                vec![Cmd::None]
            }

            Msg::KeyPressed(key_event) => self.handle_key_input(key_event),

            Msg::FocusNextPane => {
                self.model.ui.focus_next_pane();
                vec![Cmd::None]
            }

            Msg::FocusPrevPane => {
                self.model.ui.focus_prev_pane();
                vec![Cmd::None]
            }

            Msg::StoryList(story_msg) => story_list::update(
                &mut self.model.ui.story_list,
                &self.model.data.stories,
                self.model.data.current_iteration.as_ref(),
                story_msg,
            ),

            Msg::Epic(_epic_msg) => {
                vec![Cmd::None]
            }

            Msg::StoriesLoaded { stories, from_cache } => {
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
            Ok(AppKey::Left) => return self.update(Msg::FocusPrevPane),
            Ok(AppKey::Right) => return self.update(Msg::FocusNextPane),
            _ => {}
        }

        match self.model.ui.focused_pane {
            PaneId::StoryList => {
                if let Some(msg) = story_list::key_to_msg(key) {
                    self.update(Msg::StoryList(msg))
                } else {
                    vec![Cmd::None]
                }
            }
            PaneId::Epic => vec![Cmd::None],
        }
    }
}
