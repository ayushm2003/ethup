mod chains;
mod cli;
mod config;
mod install;
mod layout;
mod runner;
mod status;

use clap::Parser;
use cli::{Cli, Commands};

use crate::chains::mainnet_config;
use crate::install::{download_lighthouse, download_reth, ensure_jwt};
use crate::layout::{bin_dir, log_dir};
use crate::runner::{spawn_cl, spawn_el, start_nodes};
use crate::status::status;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Cli::parse();

    match args.command {
        Commands::Run { quiet } => {
            let bin_dir = bin_dir();
            if !bin_dir.join("reth").exists() {
                download_reth().await?;
            }

            if !bin_dir.join("lighthouse").exists() {
                download_lighthouse().await?;
            }

            let _ = ensure_jwt().await?;
            let (el_hoodi, cl_hoodi) = mainnet_config();
            let mut el = spawn_el(&el_hoodi, quiet)?;
            let mut cl = spawn_cl(&cl_hoodi, quiet)?;

            if quiet {
                println!("Running quietly. Logs at {}", log_dir().display());
            }

            start_nodes(&mut el, &mut cl, quiet).await?;
        }
        Commands::Status => {
            let (el, cl) = mainnet_config();
            status(&el, &cl).await?;
        }
    }

    Ok(())
}
