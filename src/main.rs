use clap::Parser;
use arc::{
    cache::Cache,
    cli::{Cli, Commands},
    worktree::check_worktree_dependencies,
};
use arc_core::Config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    check_worktree_dependencies()?;

    let args = Cli::parse();
    if let Some(cmd) = args.command {
        // Migrate runs before config is loaded, since it exists precisely
        // to move config into place.
        if matches!(cmd, Commands::Migrate) {
            arc::migrate::migrate()?;
            return Ok(());
        }

        let config = Config::read()?;
        let cache = Cache::read(config.cache_dir.clone()).await;
        arc::handle_command(cmd, cache, &config).await?;
        config.write()?;
        return Ok(());
    }

    // need to do the ratatui stuff manually since we are using await in the main
    let mut terminal = ratatui::init();
    let result = arc::run(&mut terminal).await;
    ratatui::restore();

    result?;
    Ok(())
}
