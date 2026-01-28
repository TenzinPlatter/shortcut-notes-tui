use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "note")]
#[command(about = "Note manager with shortcut integration", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    Open,
}

