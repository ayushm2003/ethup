mod cli;
mod runner;

use clap::Parser;
use cli::{Cli, Commands};

use crate::runner::{bin_dir, download_lighthouse, download_reth, ensure_jwt};

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

            tokio::try_join!(
                runner::run_reth(&jwt_path),
                runner::run_lighthouse(&jwt_path),
            )?;
        }
        Commands::Status => println!("printing status"),
    }

    Ok(())
}
