use clap::Parser;
use jired::{Args, app::Jira, error::Result};

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    Jira::new().await.init().await?;

    Ok(())
}
