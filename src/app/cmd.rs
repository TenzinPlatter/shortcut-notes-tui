use std::io::Stdout;

use anyhow::Result;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::ExecutableCommand;
use ratatui::{Terminal, prelude::CrosstermBackend};
use tokio::sync::mpsc::UnboundedSender;
use std::{env, fs::{create_dir_all, read_to_string, OpenOptions}, process::Command as ProcessCommand};

use crate::{
    api::{iteration::Iteration, story::Story, ApiClient},
    app::msg::Msg,
    cache::Cache,
    config::Config,
    dbg_file,
    note::Note,
};

#[derive(Debug, Clone)]
pub enum Cmd {
    None,
    OpenNote { story: Story, iteration: Option<Iteration> },
    WriteCache,
    FetchStories { iteration: Iteration },
    FetchEpics,
    Batch(Vec<Cmd>),
}

pub async fn execute(
    cmd: Cmd,
    sender: UnboundedSender<Msg>,
    config: &Config,
    cache: &mut Cache,
    api_client: &ApiClient,
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
) -> Result<()> {
    match cmd {
        Cmd::None => Ok(()),

        Cmd::OpenNote { story, iteration } => {
            open_note_in_editor(story, iteration, config, terminal)?;
            sender.send(Msg::NoteOpened).ok();
            Ok(())
        }

        Cmd::WriteCache => {
            cache.write()?;
            sender.send(Msg::CacheWritten).ok();
            Ok(())
        }

        Cmd::FetchStories { iteration } => {
            let sender = sender.clone();
            let api_client = api_client.clone();

            tokio::spawn(async move {
                match api_client.get_owned_iteration_stories(&iteration).await {
                    Ok(stories) => {
                        sender.send(Msg::StoriesLoaded { stories, from_cache: false }).ok();
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

        Cmd::Batch(commands) => {
            for cmd in commands {
                Box::pin(execute(cmd, sender.clone(), config, cache, api_client, terminal)).await?;
            }
            Ok(())
        }
    }
}

fn open_note_in_editor(
    story: Story,
    iteration: Option<Iteration>,
    config: &Config,
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
) -> Result<()> {
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

    std::io::stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;

    ProcessCommand::new(editor).arg(note.path).status()?;

    std::io::stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    terminal.clear()?;

    Ok(())
}
