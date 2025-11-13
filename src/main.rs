mod cli;
mod runner;

use clap::Parser;
use cli::{Cli, Commands};

use crate::runner::{download_lighthouse, download_reth, bin_dr};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Cli::parse();

    match args.command {
        Commands::Run => {
            match runner::check_reth().await {
                Ok(_) => unimplemented!(),
                Err(e)
                    if e.downcast_ref::<std::io::Error>()
                        .map(|io| io.kind() == std::io::ErrorKind::NotFound)
                        .unwrap_or(false) =>
                {
                    if !bin_dr().join("reth").exists() {
						download_reth().await?
					}
                }
                Err(e) => return Err(e),
            };

            match runner::check_lighthouse().await {
                Ok(_) => unimplemented!(),
                Err(e)
                    if e.downcast_ref::<std::io::Error>()
                        .map(|io| io.kind() == std::io::ErrorKind::NotFound)
                        .unwrap_or(false) =>
                {
                    if !bin_dr().join("lighthouse").exists() {
						download_lighthouse().await?
					}
                }
                Err(e) => return Err(e),
            };
        }
        Commands::Status => println!("printing status"),
    }

    Ok(())
}
