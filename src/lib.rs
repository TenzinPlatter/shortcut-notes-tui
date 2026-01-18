use ratatui::layout::Direction;

use crate::{
    api::ApiClient,
    error_display::ErrorExt,
    view::{View, ViewBuilder},
};

pub mod api;
pub mod app;
pub mod error_display;
pub mod keys;
pub mod pane;
pub mod view;

pub async fn get_main_view(api_client: ApiClient) -> error_display::Result<View> {
    let epic_view = api_client.get_epics_view().await?;

    Ok(ViewBuilder::default()
        .add_non_selectable(epic_view)
        .direction(Direction::Vertical)
        .build())
}

pub async fn get_api_key() -> error_display::Result<String> {
    std::env::var("SHORTCUT_API_TOKEN").blocking().with_message(
        "Please set the SHORTCUT_API_TOKEN environment variable to authenticate with Shortcut",
    )
}

pub async fn get_user_id() -> error_display::Result<String> {
    // TODO: maybe fetch this from the API using the token instead of env var
    std::env::var("SHORTCUT_USER_ID").blocking().with_message(
        "Please set the SHORTCUT_USER_ID environment variable to authenticate with Shortcut",
    )
}
