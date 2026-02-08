use clap::Parser;
use shortcut_notes::{
    cache::Cache, cli::Cli, config::Config, worktree::check_worktree_dependencies,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    check_worktree_dependencies()?;

    let args = Cli::parse();
    if let Some(cmd) = args.command {
        let config = Config::read()?;
        let cache = Cache::read(config.cache_dir.clone());
        shortcut_notes::handle_command(cmd, cache, &config).await?;
        config.write()?;
        return Ok(());
    }

    // need to do the ratatui stuff manually since we are using await in the main
    let mut terminal = ratatui::init();
    let result = shortcut_notes::run(&mut terminal).await;
    ratatui::restore();

    result?;
    Ok(())
}
