use std::{
    env,
    io::{Stdout, stdout},
    process::Command,
};

use anyhow::Result;
use crossterm::{
    ExecutableCommand,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, prelude::CrosstermBackend};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    api::{
        epic::{Epic, view::create_epics_view},
        iteration::Iteration,
        story::Story,
    },
    app::App,
    pane::ListPane,
    view::ViewBuilder,
};

/// Events sent from background tasks to the main app
pub enum AppEvent {
    UnexpectedError(anyhow::Error),
    EpicsLoaded(Vec<Epic>),
    /// (stories, are_saved)
    StoriesLoaded((Vec<Story>, bool)),
    IterationLoaded(Iteration),
    OpenInEditor(String),
}

impl App {
    pub(super) fn handle_app_event(
        &mut self,
        event: AppEvent,
        sender: UnboundedSender<AppEvent>,
        terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    ) -> Result<()> {
        match event {
            AppEvent::UnexpectedError(e) => {
                return Err(e);
            }
            AppEvent::EpicsLoaded(epics) => {
                self.view = create_epics_view(epics);
            }
            AppEvent::StoriesLoaded((stories, are_saved)) => {
                if !are_saved
                    && let Some(saved) = &self.cache.iteration_stories
                    && saved.iter().zip(stories.iter()).all(|(a, b)| a.id == b.id)
                {
                    return Ok(());
                }

                self.cache.iteration_stories = Some(stories.clone());
                self.cache.write()?;
                self.view = ViewBuilder::default()
                    .add_selectable(ListPane::new(stories, sender))
                    .build();
            }
            AppEvent::IterationLoaded(iteration) => {
                self.cache.current_iteration = Some(iteration);
                self.cache.write()?;
                self.view = App::get_loading_view_stories();
            }
            AppEvent::OpenInEditor(file) => {
                let editor = env::var("EDITOR").unwrap_or("nvim".to_string());
                let iteration_name = match &self.cache.current_iteration {
                    Some(it) => it.name.clone(),
                    None => "No Iteration".to_string(),
                };

                let note_path = format!("{}/{}/{}", self.config.notes_dir, iteration_name, file);

                stdout().execute(LeaveAlternateScreen)?;
                disable_raw_mode()?;

                Command::new(editor).arg(note_path).status()?;

                stdout().execute(EnterAlternateScreen)?;
                enable_raw_mode()?;
                terminal.clear()?;
            }
        }

        Ok(())
    }
}
