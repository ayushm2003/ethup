mod cli;
mod install;
mod layout;
mod runner;

use clap::Parser;
use cli::{Cli, Commands};

use crate::install::{download_lighthouse, download_reth, ensure_jwt};
use crate::layout::bin_dir;
use crate::runner::{spawn_lighthouse, spawn_reth, start_nodes};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Cli::parse();

    match args.command {
        Commands::Run => {
            let bin_dir = bin_dir();
            if !bin_dir.join("reth").exists() {
                download_reth().await?;
            }

            if !bin_dir.join("lighthouse").exists() {
                download_lighthouse().await?;
            }

            let jwt_path = ensure_jwt().await?;

            let mut el = spawn_reth(&jwt_path)?;
            let mut cl = spawn_lighthouse(&jwt_path)?;

            start_nodes(&mut el, &mut cl).await?;
        }
        Commands::Status => println!("printing status"),
    }

    Ok(())
}
