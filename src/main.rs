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
        cli::Commands::Start {
            session_name,
            layout,
            detach,
        } => {
            commands::start::execute(session_name, layout, detach)?;
            Ok(())
        }
        cli::Commands::Attach { session_name, list } => {
            commands::attach::execute(session_name, list)?;
            Ok(())
        }
        cli::Commands::Kill {
            session_name,
            force,
            all,
        } => {
            commands::kill::execute(session_name, force, all)?;
            Ok(())
        }
        cli::Commands::Send {
            command,
            args,
            timeout,
            work_dir,
            env_vars,
            sequential,
        } => {
            commands::send::execute(
                &command,
                &args,
                timeout,
                work_dir.as_deref(),
                &env_vars,
                sequential,
            )?;
            Ok(())
        }
        cli::Commands::Hey {
            agent,
            command,
            args,
            timeout,
            work_dir,
            env_vars,
            interactive,
        } => {
            commands::hey::execute(
                &agent,
                &command,
                &args,
                timeout,
                work_dir.as_deref(),
                &env_vars,
                interactive,
            )?;
            Ok(())
        }
        cli::Commands::Sync {
            agent,
            force,
            strategy,
            dry_run,
        } => {
            commands::sync::execute(agent.as_deref(), force, &strategy, dry_run)?;
            Ok(())
        }
        cli::Commands::Remove {
            agent,
            force,
            keep_workspace,
            merge_branch,
        } => {
            commands::remove::execute(&agent, force, keep_workspace, merge_branch)?;
            Ok(())
        }
        cli::Commands::Warp {
            workspace,
            list,
            print,
            relative,
        } => {
            commands::warp::execute(workspace, list, print, relative)?;
            Ok(())
        }
        cli::Commands::Zoom {
            agent,
            unzoom,
            list,
        } => {
            commands::zoom::execute(agent, unzoom, list)?;
            Ok(())
        }
        cli::Commands::Status {
            session_name,
            cleanup,
            detailed,
        } => {
            commands::status::execute(session_name, cleanup, detailed)?;
            Ok(())
        }
        cli::Commands::Help {
            command,
            search,
            patterns,
            troubleshooting,
            quick,
            accessible,
            category,
        } => {
            let args = commands::help::HelpArgs {
                command,
                search,
                patterns,
                troubleshooting,
                quick,
                accessible,
                category,
            };
            commands::help::execute(args)?;
            Ok(())
        }
    }
}
