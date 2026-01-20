use anyhow::Context;
use ratatui::DefaultTerminal;
use uuid::Uuid;

use crate::app::App;

pub mod api;
pub mod app;
pub mod config;
pub mod keys;
pub mod pane;
pub mod view;

pub async fn get_api_key() -> anyhow::Result<String> {
    std::env::var("SHORTCUT_API_TOKEN").context(
        "Please set the SHORTCUT_API_TOKEN environment variable to authenticate with Shortcut",
    )
}

pub async fn get_user_id(saved_user_id: Option<Uuid>) -> anyhow::Result<String> {
    if let Some(id) = saved_user_id {
        id
    } else {
        // TODO: fetch user id from members endpoint
    }
    // TODO: maybe fetch this from the API using the token instead of env var
    std::env::var("SHORTCUT_USER_ID").context(
        "Please set the SHORTCUT_USER_ID environment variable to authenticate with Shortcut",
    )
}

pub async fn run(terminal: &mut DefaultTerminal) -> anyhow::Result<()> {
    let mut app = App::init().await?;
    app.main_loop(terminal).await?;

    Ok(())
}
