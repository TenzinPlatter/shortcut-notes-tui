use anyhow::Result;

use crate::{
    api::{
        epic::{view::create_epics_view, Epic},
        iteration::Iteration,
        story::Story,
    }, app::App, pane::ListPane, view::ViewBuilder
};

/// Events sent from background tasks to the main app
pub enum AppEvent {
    UnexpectedError(anyhow::Error),
    EpicsLoaded(Vec<Epic>),
    /// (stories, are_saved)
    StoriesLoaded((Vec<Story>, bool)),
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
            AppEvent::StoriesLoaded((stories, are_saved)) => {
                // TODO: implement eq traits for &Story so we don't have to clone
                if !are_saved && let Some(saved) = &self.config.iteration_stories
                    && saved.iter().zip(stories.clone()).all(|(a, b)| *a == b)
                {
                    return Ok(());
                }

                self.config.iteration_stories = Some(stories.clone());
                self.config.write()?;
                self.view = ViewBuilder::default()
                    .add_selectable(ListPane::new(stories))
                    .build();
            }
            AppEvent::IterationLoaded(iteration) => {
                self.config.current_iteration = Some(iteration);
                self.config.write()?;
                self.view = App::get_loading_view_stories();
            }
        }

        Ok(())
    }
}
