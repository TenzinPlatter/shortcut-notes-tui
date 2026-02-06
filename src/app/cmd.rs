use std::io::{Stdout, Write};

use anyhow::Result;
use crossterm::ExecutableCommand;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::{Terminal, prelude::CrosstermBackend};
use std::{
    fs::{OpenOptions, create_dir_all, read_to_string},
    process::Command as ProcessCommand,
};
use tempfile::NamedTempFile;
use tokio::sync::mpsc::UnboundedSender;

use crate::app::model::Model;
use crate::error::ErrorInfo;
use crate::tmux::{session_attach, session_create};
use crate::{
    api::{ApiClient, iteration::Iteration, story::Story},
    app::msg::Msg,
    config::Config,
    dbg_file,
    note::Note,
};

#[derive(Debug, Clone)]
pub enum Cmd {
    None,
    OpenNote {
        story: Story,
        iteration: Option<Iteration>,
    },
    WriteCache,
    FetchStories {
        iteration: Iteration,
    },
    EditStoryContent(Story),
    FetchEpics,
    SelectStory(Option<Story>),
    OpenTmuxSession {
        story_name: String,
    },
    Batch(Vec<Cmd>),
}

pub async fn execute(
    cmd: Cmd,
    sender: UnboundedSender<Msg>,
    model: &mut Model,
    api_client: &ApiClient,
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
) -> Result<()> {
    match cmd {
        Cmd::None => Ok(()),

        Cmd::OpenNote { story, iteration } => {
            open_note_in_editor_tui(story.clone(), iteration, &model.config, terminal)?;
            sender.send(Msg::NoteOpened).ok();
            Ok(())
        }

        Cmd::WriteCache => {
            dbg_file!("Writing cache with: {:?}", model.cache.active_story);
            model.cache.write()?;
            sender.send(Msg::CacheWritten).ok();
            Ok(())
        }

        Cmd::FetchStories { iteration } => {
            let sender = sender.clone();
            let api_client = api_client.clone();

            tokio::spawn(async move {
                match api_client.get_owned_iteration_stories(&iteration).await {
                    Ok(stories) => {
                        sender
                            .send(Msg::StoriesLoaded {
                                stories,
                                from_cache: false,
                            })
                            .ok();
                    }
                    Err(e) => {
                        let info = ErrorInfo::new(
                            "Failed to get stories for current iteration".to_string(),
                            e.to_string(),
                        );

                        sender.send(Msg::Error(info)).ok();
                    }
                }
            });
            Ok(())
        }

        Cmd::FetchEpics => {
            dbg_file!("FetchEpics not yet implemented");
            Ok(())
        }

        Cmd::SelectStory(story) => {
            if let Some(active_story) = &model.ui.story_list.active_story
                && let Some(story) = &story
                && active_story.id == story.id
            {
                model.cache.active_story = None;
                model.ui.story_list.active_story = None;
            } else {
                model.ui.story_list.active_story = story.clone();
                model.cache.active_story = story;
            }

            Ok(())
        }

        Cmd::Batch(commands) => {
            for cmd in commands {
                Box::pin(execute(cmd, sender.clone(), model, api_client, terminal)).await?;
            }

            Ok(())
        }

        Cmd::OpenTmuxSession { story_name } => {
            let session_name = Story::tmux_session_name(&story_name);
            open_tmux_session(&session_name).await?;
            Ok(())
        }

        Cmd::EditStoryContent(story) => {
            // NOTE: this only works for editors that run from their process, i.e. code spawns the
            // vscode gui, then ends itself, will not work as it is now
            let mut tempfile = NamedTempFile::new()?;
            tempfile.write_all(story.description.as_bytes())?;

            let tmp_path = tempfile.path();

            std::io::stdout().execute(LeaveAlternateScreen)?;
            disable_raw_mode()?;

            ProcessCommand::new(&model.config.editor)
                .arg(tmp_path)
                .status()?;

            std::io::stdout().execute(EnterAlternateScreen)?;
            enable_raw_mode()?;
            terminal.clear()?;

            // TODO: update story description

            Ok(())
        }
    }
}

pub fn open_note_in_editor(
    story: Story,
    iteration: Option<Iteration>,
    config: &Config,
) -> anyhow::Result<()> {
    let note = Note::new(&config.notes_dir, &story, iteration.as_ref());

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

    ProcessCommand::new(&config.editor)
        .arg(note.path)
        .status()?;

    Ok(())
}

pub fn open_note_in_editor_tui(
    story: Story,
    iteration: Option<Iteration>,
    config: &Config,
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
) -> anyhow::Result<()> {
    std::io::stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;

    let result = open_note_in_editor(story, iteration, config);

    std::io::stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    terminal.clear()?;

    result
}

pub async fn open_tmux_session(name: &str) -> anyhow::Result<()> {
    session_create(name).await?;
    session_attach(name).await?;
    Ok(())
}
