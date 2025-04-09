use anyhow::Result;
use jired::app::Jira;

#[tokio::main]
async fn main() -> Result<()> {
    Jira::new().await.init().await?;

    Ok(())
}
