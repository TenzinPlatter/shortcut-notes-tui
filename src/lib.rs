use ratatui::DefaultTerminal;
use uuid::Uuid;

use crate::{
    api::{story::Story, user::get_user_id_from_api},
    app::{
        App,
        cmd::{open_note_in_editor, open_tmux_session},
    },
    cache::Cache,
    cli::Commands,
    config::Config,
};

pub mod api;
pub mod app;
pub mod cache;
pub mod cli;
pub mod config;
pub mod dummy;
pub mod error;
pub mod keys;
pub mod macros;
pub mod note;
pub mod text_utils;
pub mod tmux;
pub mod view;

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

    Ok(())
}

pub async fn handle_command(command: Commands, cache: Cache, config: Config) -> anyhow::Result<()> {
    match command {
        Commands::Open => {
            if let Some(story) = cache.active_story {
                open_note_in_editor(story, cache.current_iteration, &config)?;
            } else {
                anyhow::bail!("You do not have a currently active story");
            }
        }

        Commands::Tmux => {
            if let Some(story) = &cache.active_story {
                let session_name = Story::tmux_session_name(&story.name);
                open_tmux_session(&session_name).await?;
            }
        }
    }

    Ok(())
}
