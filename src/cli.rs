use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "arc")]
#[command(about = "Note manager with shortcut integration", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(alias = "n")]
    Note,

    #[command(alias = "t")]
    Tmux,

    #[command(alias = "clear")]
    ClearCache,

    #[command(alias = "o")]
    Open,

    #[command()]
    Cat,

    /// Migrate config, cache, and notes directories from the old `shortcut-notes` name to `arc`.
    #[command()]
    Migrate,
}
