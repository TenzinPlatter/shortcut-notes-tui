use shortcut_notes_tui::app::App;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut terminal = ratatui::init();
    let result = App::run_with_error_handling(&mut terminal).await;
    ratatui::restore();
    result
}
