use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "eth", about = "ethereum made simple")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Run,
    Status,
}
