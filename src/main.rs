use anyhow::Result;
use clap::Parser;

mod cli;
mod commands;
mod config;
mod error;
mod models;
mod utils;
mod validation;

#[derive(Parser)]
#[command(name = "sprite")]
#[command(about = "A robust command-line toolkit for managing multiple AI coding agents")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: cli::Commands,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        cli::Commands::Init { force, agents } => {
            let options = commands::init::InitOptions { force, agents };
            commands::init::execute(options)?;
            Ok(())
        }
        cli::Commands::Config { command } => {
            commands::config::execute(command)?;
            Ok(())
        }
        cli::Commands::Agents { command } => {
            commands::agents::execute(command)?;
            Ok(())
        }
        cli::Commands::Start { session_name, layout, detach } => {
            commands::start::execute(session_name, layout, detach)?;
            Ok(())
        }
        cli::Commands::Attach { .. } => commands::attach::execute(),
        cli::Commands::Kill { .. } => commands::kill::execute(),
        cli::Commands::Send { .. } => commands::send::execute(),
        cli::Commands::Hey { .. } => commands::hey::execute(),
        cli::Commands::Sync { .. } => commands::sync::execute(),
        cli::Commands::Remove { .. } => commands::remove::execute(),
        cli::Commands::Warp { .. } => commands::warp::execute(),
        cli::Commands::Zoom { .. } => commands::zoom::execute(),
    }
}
