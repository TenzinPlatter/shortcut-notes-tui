use std::io::{Stdout, Write};

use anyhow::Result;
use crossterm::ExecutableCommand;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::{Terminal, prelude::CrosstermBackend};
use std::{
    fs::{OpenOptions, create_dir_all, read_to_string},
    process::Command,
};
use tempfile::NamedTempFile;
use tokio::sync::mpsc::UnboundedSender;

use crate::app::model::Model;
use crate::error::ErrorInfo;
use crate::tmux::{session_attach, session_create};
use crate::worktree::{create_worktree, get_repo_list, select_repo_with_fzf};
use crate::{
    api::{ApiClient, story::Story},
    app::msg::Msg,
    config::Config,
    dbg_file,
    note::Note,
};

#[derive(Debug, Clone)]
pub enum Cmd {
    None,
    OpenNote {
        story_id: i32,
        story_name: String,
        story_app_url: String,
        iteration_app_url: Option<String>,
    },
    WriteCache,
    FetchStories {
        iteration_ids: Vec<i32>,
    },
    EditStoryContent {
        story_id: i32,
        description: String,
    },
    FetchEpics,
    SelectStory(Option<Story>),
    ActionMenuVisibility(bool),
    CreateGitWorktree {
        branch_name: String,
    },
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

        Cmd::OpenNote {
            story_id,
            story_name,
            story_app_url,
            iteration_app_url,
        } => {
            open_note_in_editor_tui(
                story_id,
                story_name,
                story_app_url,
                iteration_app_url,
                &model.config,
                terminal,
            )?;
            sender.send(Msg::NoteOpened).ok();
            Ok(())
        }

        Cmd::WriteCache => {
            dbg_file!("Writing cache with: {:?}", model.cache.active_story);
            model.cache.write()?;
            sender.send(Msg::CacheWritten).ok();
            Ok(())
        }

        Cmd::FetchStories { iteration_ids } => {
            let sender = sender.clone();
            let api_client = api_client.clone();

            tokio::spawn(async move {
                match api_client.get_owned_iteration_stories(iteration_ids).await {
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
            if let Some(active_story) = &model.data.active_story
                && let Some(story) = &story
                && active_story.id == story.id
            {
                model.cache.active_story = None;
                model.data.active_story = None;
            } else {
                model.data.active_story = story.clone();
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

        Cmd::EditStoryContent {
            story_id: _,
            description,
        } => {
            // NOTE: this only works for editors that run from their process, i.e. code spawns the
            // vscode gui, then ends itself, will not work as it is now
            let mut tempfile = NamedTempFile::new()?;
            tempfile.write_all(description.as_bytes())?;

            let tmp_path = tempfile.path();

            std::io::stdout().execute(LeaveAlternateScreen)?;
            disable_raw_mode()?;

            Command::new(&model.config.editor).arg(tmp_path).status()?;

            std::io::stdout().execute(EnterAlternateScreen)?;
            enable_raw_mode()?;
            terminal.clear()?;

            // TODO: update story description

            Ok(())
        }

        Cmd::ActionMenuVisibility(enabled) => {
            model.ui.action_menu.is_showing = enabled;
            if enabled {
                // Capture the currently selected story ID
                model.ui.action_menu.target_story_id = model.ui.story_list.selected_story_id;
            } else {
                model.ui.action_menu.list_state.select(Some(0));
                model.ui.action_menu.target_story_id = None;
            }

            Ok(())
        }

        Cmd::CreateGitWorktree { branch_name } => {
            let repos = get_repo_list(&model.config).await?;
            let chosen = select_repo_with_fzf(&repos, terminal)?;
            let path = model.config.repositories_directory.join(chosen);
            create_worktree(&path, &branch_name).await?;

            Ok(())
        }
    }
}

pub fn open_note_in_editor(
    story_id: i32,
    story_name: String,
    story_app_url: String,
    iteration_app_url: Option<String>,
    config: &Config,
) -> anyhow::Result<()> {
    let note = Note::new(
        &config.notes_dir,
        story_id,
        story_name,
        story_app_url,
        iteration_app_url,
    );

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

    Command::new(&config.editor).arg(note.path).status()?;

    Ok(())
}

pub fn open_note_in_editor_tui(
    story_id: i32,
    story_name: String,
    story_app_url: String,
    iteration_app_url: Option<String>,
    config: &Config,
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
) -> anyhow::Result<()> {
    std::io::stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;

    let result = open_note_in_editor(
        story_id,
        story_name,
        story_app_url,
        iteration_app_url,
        config,
    );

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
