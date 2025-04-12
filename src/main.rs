use jired::app::Jira;
use jired::error::Result;

#[tokio::main]
async fn main() -> Result<()> {
    Jira::new().await.init().await?;

    Ok(())
}
