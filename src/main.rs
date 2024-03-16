use std::path::PathBuf;

use clap::Parser;
use clap_derive::{Parser, Subcommand};

mod commands;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Pack {},
}

fn main() -> anyhow::Result<()> {
    simple_logger::init()?;

    let cli = Cli::parse();
    let result = match &cli.command {
        Commands::Pack {} => {
            let path;
            if cfg!(debug_assertions) {
                path = PathBuf::from("E:\\org\\vertexscripts\\scripts\\vx_reports");
            } else {
                path = PathBuf::from(".");
            }

            commands::pack::handle_command(path)
        }
    };

    match result {
        Ok(_) => log::info!("Command executed successfully"),
        Err(e) => log::error!("Error: {}", e),
    }

    Ok(())
}
