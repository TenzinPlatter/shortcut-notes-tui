use anyhow::Context;
use ratatui::DefaultTerminal;
use uuid::Uuid;

use crate::{api::{user::get_user_id_from_api, ApiClient}, app::App};

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

pub async fn get_user_id(
    saved_user_id: Option<Uuid>,
    api_token: &str
) -> anyhow::Result<Uuid> {
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
