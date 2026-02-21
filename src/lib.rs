use std::{fs, process::Command};

use anyhow::Context;
use ratatui::DefaultTerminal;
use uuid::Uuid;

use crate::{
    api::{
        story::{Story, get_story_associated_iteration},
        user::get_user_id_from_api,
    },
    app::{
        App,
        cmd::{open_note_in_editor, open_tmux_session},
    },
    cache::Cache,
    cli::Commands,
    config::Config,
    note::Note,
};

pub mod api;
pub mod app;
pub mod cache;
pub mod cli;
pub mod config;
pub mod custom_list;
pub mod dummy;
pub mod error;
pub mod keybindings;
pub mod macros;
pub mod note;
pub mod text_utils;
pub mod time;
pub mod tmux;
pub mod view;
pub mod worktree;
#[macro_use]
pub mod keys;

macro_rules! no_active_story {
    () => {
        anyhow::bail!("You do not have a currently active story");
    };
}

pub async fn get_user_id(saved_user_id: Option<Uuid>, api_token: &str) -> anyhow::Result<Uuid> {
    let id = if let Some(id) = saved_user_id {
        id
    } else {
        get_user_id_from_api(api_token).await?
    };

    Ok(id)
}

pub async fn run(terminal: &mut DefaultTerminal) -> anyhow::Result<()> {
    let mut app = App::init().await?;
    app.main_loop(terminal).await?;
    app.config.write()?;

    Ok(())
}

pub async fn handle_command(
    command: Commands,
    cache: Cache,
    config: &Config,
) -> anyhow::Result<()> {
    match command {
        Commands::Note => {
            if let Some(story) = &cache.active_story {
                let iteration_app_url = cache
                    .current_iterations_ref()
                    .and_then(|iterations| {
                        get_story_associated_iteration(story.iteration_id, iterations)
                    })
                    .map(|it| it.app_url.clone());

                open_note_in_editor(
                    story.id,
                    story.name.clone(),
                    story.app_url.clone(),
                    iteration_app_url,
                    config,
                )?;

                Ok(())
            } else {
                no_active_story!();
            }
        }

        Commands::Tmux => {
            if let Some(story) = &cache.active_story {
                let session_name = Story::tmux_session_name(&story.name);
                open_tmux_session(&session_name).await?;
                Ok(())
            } else {
                no_active_story!();
            }
        }

        Commands::ClearCache => {
            let cache_file = Cache::get_cache_file(config.cache_dir.clone());
            fs::remove_file(cache_file)?;
            Ok(())
        }

        Commands::Open => {
            if let Some(story) = &cache.active_story {
                open::that(story.app_url.clone())
                    .with_context(|| format!("Failed to open {}", story.app_url))
            } else {
                no_active_story!();
            }
        }

        Commands::Cat => {
            if let Some(story) = &cache.active_story {
                let iteration_app_url = cache
                    .current_iterations_ref()
                    .and_then(|iterations| {
                        get_story_associated_iteration(story.iteration_id, iterations)
                    })
                    .map(|it| it.app_url.clone());

                let note = Note::new(
                    &config.notes_dir,
                    story.id,
                    story.name.clone(),
                    story.app_url.clone(),
                    iteration_app_url,
                );

                let status = Command::new("cat").arg(note.path).status()?;

                if !status.success() {
                    anyhow::bail!("Failed to cat not for story {}", &story.name);
                }

                Ok(())
            } else {
                no_active_story!();
            }
        }
    }
}
