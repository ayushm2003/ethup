use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "eth", version, about = "ethereum made simple")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Run {
		#[arg(short, long)]
		quiet: bool,
	},
    Status,
}
