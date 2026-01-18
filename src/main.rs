use shortcut_notes_tui::app::App;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // need to do the ratatui stuff manually since we are using await in the main
    let mut terminal = ratatui::init();
    let result = App::run(&mut terminal).await;
    ratatui::restore();

    result?;
    Ok(())
}
