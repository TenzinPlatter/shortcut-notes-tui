use std::io::Stdout;

use anyhow::Result;
use crossterm::ExecutableCommand;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::{Terminal, prelude::CrosstermBackend};
use std::{
    env,
    fs::{OpenOptions, create_dir_all, read_to_string},
    process::Command as ProcessCommand,
};
use tokio::sync::mpsc::UnboundedSender;

use crate::app::model::Model;
use crate::{
    api::{ApiClient, iteration::Iteration, story::Story},
    app::msg::Msg,
    cache::Cache,
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
    FetchEpics,
    SelectStory(Option<Story>),
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
                        sender.send(Msg::Error(e.to_string())).ok();
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
    }
}

pub fn open_note_in_editor(
    story: Story,
    iteration: Option<Iteration>,
    config: &Config,
) -> anyhow::Result<()> {
    let note = Note::new(&config.notes_dir, &story, iteration.as_ref());

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

    ProcessCommand::new(editor).arg(note.path).status()?;

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
