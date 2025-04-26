use clap::Parser;
use jired::{
    Args,
    boards::Board,
    boards::jira::Jira,
    clocks::{Clock, clockify::ClockifyClock},
    common::config::ConfigParser,
    error::Result,
    tracing::Tracer,
};

#[tokio::main]
async fn main() -> Result<()> {
    Tracer::init()?;
    let args = Args::parse();

    // commented to test the clockify
    //Jira::new().await.init(args).await?;
    //
    ConfigParser::parse().await?;
    //ClockifyClock::new().await.init().await?;
    Ok(())
}
