mod cli;
mod runner;

use clap::Parser;
use cli::{Cli, Commands};

use crate::runner::{bin_dir, download_lighthouse, download_reth};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Cli::parse();

    match args.command {
        Commands::Run => {
			let bin_dir = bin_dir();
            if !bin_dir.join("reth").exists() {
                download_reth().await?
            }

            if !bin_dir.join("lighthouse").exists() {
                download_lighthouse().await?
            }
        }
        Commands::Status => println!("printing status"),
    }

    Ok(())
}
