#![cfg(any())]

use std::{
    env,
    fs::{OpenOptions, create_dir_all, read_to_string},
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
    dbg_file,
    note::Note,
    pane::StoryListPane,
    view::ViewBuilder,
};

/// Events sent from background tasks to the main app
pub enum AppEvent {
    UnexpectedError(anyhow::Error),
    EpicsLoaded(Vec<Epic>),
    /// (stories, are_saved)
    StoriesLoaded((Vec<Story>, bool)),
    IterationLoaded(Iteration),
    OpenStoryNote(Story),
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
                    .add_selectable(StoryListPane::new(stories, sender))
                    .build();
            }
            AppEvent::IterationLoaded(iteration) => {
                self.cache.current_iteration = Some(iteration);
                self.cache.write()?;
                self.view = App::get_loading_view_stories();
            }
            AppEvent::OpenStoryNote(story) => {
                // TODO: epic from story
                let note = Note::new(
                    &self.config.notes_dir,
                    &story,
                    self.cache.current_iteration.as_ref(),
                );

                let editor = env::var("EDITOR").unwrap_or("nvim".to_string());

                if note.path.is_dir() {
                    anyhow::bail!("Note path: {} is not a file", note.path.display());
                }

                if let Some(p) = note.path.parent() {
                    create_dir_all(p)?;
                }

                dbg_file!("Opening note: {}", note.path.display());

                let mut f = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .read(true)
                    .open(&note.path)?;
                let buf = read_to_string(&note.path)?;

                if buf.is_empty() {
                    dbg_file!("Writing frontmatter to {}", note.path.display());
                    note.write_frontmatter(&mut f)?;
                }

                stdout().execute(LeaveAlternateScreen)?;
                disable_raw_mode()?;

                Command::new(editor).arg(note.path).status()?;

                stdout().execute(EnterAlternateScreen)?;
                enable_raw_mode()?;
                terminal.clear()?;
            }
        }

        Ok(())
    }
}
