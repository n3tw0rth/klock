mod boards;
mod cli;
mod clocks;
mod commands;
mod config;
mod error;
mod session;

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
        Commands::Auth { action } => commands::auth::handle(action).await?,
        Commands::Add {
            project_code,
            search_string,
        } => commands::add::handle(project_code, search_string).await?,
        Commands::Start {
            project_code,
            search_string,
            start,
            end,
        } => commands::start::handle(project_code, search_string, start, end).await?,
        Commands::Stop { at } => commands::stop::handle(at).await?,
        Commands::Set { date } => commands::set::handle(date).await?,
        Commands::Log { summary } => commands::log::handle(summary).await?,
        Commands::Pending { action } => commands::pending::handle(action).await?,
    }
    Ok(())
}
