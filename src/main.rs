use clap::Parser;
use jired::tracing::Tracer;
use jired::{Args, boards::Board, boards::jira::Jira, error::Result};

#[tokio::main]
async fn main() -> Result<()> {
    Tracer::init()?;
    let args = Args::parse();

    Jira::new().await.init(args).await?;

    Ok(())
}
