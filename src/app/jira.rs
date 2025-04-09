use anyhow::Result;
use keyring::Entry;
use std::io;

#[derive(Default)]
pub struct Jira {
    pub username: String,
    pub jira_api_token: String,
    pub clockify_api_token: String,
}

impl Jira {
    pub async fn new() -> Self {
        Self { ..Jira::default() }
    }

    pub async fn init(mut self) -> Result<()> {
        if !self.clockify_api_token.is_empty() {
            println!("{}", self.clockify_api_token);
        } else {
            self.set_credentials().await?;
        }

        Ok(())
    }

    pub async fn set_credentials(&mut self) -> Result<()> {
        io::stdin()
            .read_line(&mut self.username)
            .expect("Failed to get the username");

        io::stdin()
            .read_line(&mut self.jira_api_token)
            .expect("Failed to get the Jira api token");

        io::stdin()
            .read_line(&mut self.clockify_api_token)
            .expect("Failed to get the Clockify api token");

        Entry::new("jired", "username")?.set_secret(self.username.as_bytes())?;
        Entry::new("jired", "jira_api_token")?.set_secret(self.jira_api_token.as_bytes())?;
        Entry::new("jired", "clockify_api_token")?
            .set_secret(self.clockify_api_token.as_bytes())?;

        Ok(())
    }
}
