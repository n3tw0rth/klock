mod boards;
mod cli;
mod clocks;
mod commands;
mod config;
mod error;
mod services;
mod session;
mod tui;
mod utils;

use clap::Parser;
use cli::{Cli, Commands};
use colored::Colorize;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let result = run(cli).await;
    if let Err(e) = result {
        eprintln!("{} {}", "Error:".red().bold(), e);
        std::process::exit(1);
    }
}

async fn run(cli: Cli) -> anyhow::Result<()> {
    match cli.command {
        None => tui::run().await?,
        Some(Commands::Auth { action }) => commands::auth::handle(action).await?,
        Some(Commands::Add {
            project_code,
            search_string,
        }) => commands::add::handle(project_code, search_string).await?,
        Some(Commands::Remove { project_code }) => {
            commands::remove::handle(project_code).await?
        }
        Some(Commands::Start {
            project_code,
            search_string,
            start,
            end,
        }) => commands::start::handle(project_code, search_string, start, end).await?,
        Some(Commands::Stop { at }) => commands::stop::handle(at).await?,
        Some(Commands::Set { date }) => commands::set::handle(date).await?,
        Some(Commands::Log { summary }) => commands::log::handle(summary).await?,
        Some(Commands::Pending { action }) => commands::pending::handle(action).await?,
    }
    Ok(())
}
