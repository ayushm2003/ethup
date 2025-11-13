mod chains;
mod cli;
mod config;
mod install;
mod layout;
mod runner;

use clap::Parser;
use cli::{Cli, Commands};

use crate::chains::hoodi_config;
use crate::install::{download_lighthouse, download_reth, ensure_jwt};
use crate::layout::bin_dir;
use crate::runner::{spawn_cl, spawn_el, start_nodes};

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

            let _ = ensure_jwt().await?;
            let (el_hoodi, cl_hoodi) = hoodi_config();
            let mut el = spawn_el(&el_hoodi)?;
            let mut cl = spawn_cl(&cl_hoodi)?;

            start_nodes(&mut el, &mut cl).await?;
        }
        Commands::Status => println!("printing status"),
    }

    Ok(())
}
