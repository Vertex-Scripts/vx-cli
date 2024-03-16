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
        Commands::Pack {} => commands::pack::handle_command(PathBuf::from(
            "E:\\org\\vertexscripts\\scripts\\vx_reports",
        )),
    };

    match result {
        Ok(_) => log::info!("Command executed successfully"),
        Err(e) => log::error!("Error: {}", e),
    }

    Ok(())
}
