use clap::Parser;
use jired::{
    Args, Commands,
    boards::{Board, jira::Jira},
    clocks::{Clock, clockify::ClockifyClock},
    common::config::ConfigParser,
    error::Result,
    tracing::Tracer,
};

#[tokio::main]
async fn main() -> Result<()> {
    Tracer::init()?;
    let config = ConfigParser::parse().await?;
    let args = Args::parse();

    match args.command {
        Commands::Log => {
            let clocks = config.get_clocks()?;
            for clock in clocks {
                if clock.as_str() == "clockify" {
                    ClockifyClock::new().await.init().await?;
                }
            }
        }
        _ => {
            Jira::new().await.init(args.clone()).await?;
        }
    }

    Ok(())
}
