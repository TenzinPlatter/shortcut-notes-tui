#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // need to do the ratatui stuff manually since we are using await in the main
    let mut terminal = ratatui::init();
    let result = shortcut_notes_tui::run(&mut terminal).await;
    ratatui::restore();

    result?;
    Ok(())
}
