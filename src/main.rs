use clap::Parser;
use jired::clocks::{Clock, clockify::ClockifyClock};
use jired::tracing::Tracer;
use jired::{Args, boards::Board, boards::jira::Jira, error::Result};

#[tokio::main]
async fn main() -> Result<()> {
    Tracer::init()?;
    let args = Args::parse();

    // commented to test the clockify
    Jira::new().await.init(args).await?;

    ClockifyClock::new().await.init().await?;
    Ok(())
}
