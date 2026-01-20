use anyhow::Result;

use crate::{
    api::{
        epic::Epic,
        iteration::Iteration,
        story::{Story, view::create_stories_view},
        epic::view::create_epics_view,
    },
    app::App,
};

/// Events sent from background tasks to the main app
pub enum AppEvent {
    UnexpectedError(anyhow::Error),
    EpicsLoaded(Vec<Epic>),
    StoriesLoaded(Vec<Story>),
    IterationLoaded(Iteration),
}

impl App {
    pub(super) fn handle_app_event(&mut self, event: AppEvent) -> Result<()> {
        match event {
            AppEvent::UnexpectedError(e) => {
                return Err(e);
            }
            AppEvent::EpicsLoaded(epics) => {
                self.view = create_epics_view(epics);
            }
            AppEvent::StoriesLoaded(stories) => {
                self.config.iteration_stories = Some(stories.clone());
                self.config.write()?;
                self.view = create_stories_view(stories);
            }
            AppEvent::IterationLoaded(iteration) => {
                self.config.current_iteration = Some(iteration);
                self.config.write()?;
                self.view = App::get_loading_view();
            }
        }

        Ok(())
    }
}
