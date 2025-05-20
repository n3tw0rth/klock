use clap::Parser;
use jired::{
    boards::{jira::Jira, Board},
    clocks::{clockify::ClockifyClock, Clock},
    common::config::ConfigParser,
    error::Result,
    tracing::Tracer,
    Args,
};

#[tokio::main]
async fn main() -> Result<()> {
    Tracer::init()?;
    let config = ConfigParser::parse().await?;
    let args = Args::parse();

    Jira::new().await.init(args.clone()).await?;

    let clocks = config.get_clocks()?;
    for clock in clocks {
        if clock.as_str() == "clockify" {
            ClockifyClock::new().await.init(args.clone()).await?;
        }
    }

    Ok(())
}
