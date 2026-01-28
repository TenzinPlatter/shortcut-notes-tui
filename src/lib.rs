use ratatui::DefaultTerminal;
use uuid::Uuid;

use crate::{
    api::user::get_user_id_from_api,
    app::{
        App,
        cmd::{self, Cmd, open_note_in_editor},
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
pub mod keys;
pub mod macros;
pub mod note;
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

pub fn handle_command(command: Commands, cache: Cache, config: Config) -> anyhow::Result<()> {
    match command {
        Commands::Open => {
            if let Some(story) = cache.current_story {
                open_note_in_editor(story, cache.current_iteration, &config)?;
            } else {
                anyhow::bail!("You do not have a currently active story");
            }
        }
    }

    Ok(())
}
