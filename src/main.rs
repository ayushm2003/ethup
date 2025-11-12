mod cli;
mod runner;

use clap::Parser;
use cli::{Cli, Commands};

use crate::runner::download_reth;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Cli::parse();

    match args.command {
        Commands::Run => match runner::run_reth().await {
            Ok(_) => unimplemented!(),
            Err(e)
                if e.downcast_ref::<std::io::Error>()
                    .map(|io| io.kind() == std::io::ErrorKind::NotFound)
                    .unwrap_or(false) =>
            {
                download_reth().await?
            }
            Err(e) => return Err(e),
        },
        Commands::Status => println!("printing status"),
    }

    Ok(())
}
