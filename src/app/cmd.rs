use std::io::Stdout;

use anyhow::Result;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::ExecutableCommand;
use ratatui::{Terminal, prelude::CrosstermBackend};
use tokio::sync::mpsc::UnboundedSender;
use std::{env, fs::{create_dir_all, read_to_string, OpenOptions}, process::Command as ProcessCommand};

use crate::{
    api::{iteration::Iteration, story::Story},
    app::msg::Msg,
    cache::Cache,
    config::Config,
    dbg_file,
    note::Note,
};

/// Commands describe side effects to execute after state updates
#[derive(Debug, Clone)]
pub enum Cmd {
    None,
    OpenNote { story: Story, iteration: Option<Iteration> },
    WriteCache,
    FetchStories { iteration_id: i64 },
    FetchEpics,
    Batch(Vec<Cmd>),
}

/// Execute a command asynchronously
pub async fn execute(
    cmd: Cmd,
    sender: UnboundedSender<Msg>,
    config: &Config,
    cache: &mut Cache,
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

        Cmd::FetchStories { iteration_id } => {
            // TODO: Implement in next task - spawn async task to fetch stories
            dbg_file!("TODO: FetchStories command for iteration {}", iteration_id);
            Ok(())
        }

        Cmd::FetchEpics => {
            // TODO: Implement in next task - spawn async task to fetch epics
            dbg_file!("TODO: FetchEpics command");
            Ok(())
        }

        Cmd::Batch(commands) => {
            for cmd in commands {
                Box::pin(execute(cmd, sender.clone(), config, cache, terminal)).await?;
            }
            Ok(())
        }
    }
}

/// Open a note in the configured editor
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

    // Suspend TUI
    std::io::stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;

    // Open editor
    ProcessCommand::new(editor).arg(note.path).status()?;

    // Resume TUI
    std::io::stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    terminal.clear()?;

    Ok(())
}
