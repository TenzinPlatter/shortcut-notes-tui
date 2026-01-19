use ratatui::{layout::Direction, DefaultTerminal};

use crate::{
    api::ApiClient, app::App, error_display::{ErrorExt, ErrorHandler, ErrorSeverity}, view::{View, ViewBuilder}
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

pub async fn run(terminal: &mut DefaultTerminal) -> error_display::Result<()> {
    match App::init().await {
        Ok(mut app) => app.main_loop(terminal).await,
        Err(e) => {
            let error_handler = ErrorHandler;
            let (error_pane, severity) = error_handler.handle(&e);

            match severity {
                ErrorSeverity::Blocking => {
                    // Don't propagate errors from show_blocking_error - just try our best
                    let _ = App::show_blocking_error(terminal, error_pane);
                    Ok(()) // Exit gracefully after showing error
                }
                ErrorSeverity::Notification => {
                    // This shouldn't happen during init, but handle anyway
                    Err(e) // Propagate the error
                }
            }
        }
    }
}
