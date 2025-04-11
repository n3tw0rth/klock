use anyhow::Result;
use keyring::Entry;
use std::io::{self, Write};

use crate::common::{Secrets, helpers};

#[derive(Debug)]
enum JiraSecrets {
    Username,
    JiraApiToken,
}

impl std::fmt::Display for JiraSecrets {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}

#[derive(Default)]
pub struct Jira {
    pub authenticated: bool,
    pub username: String,
    pub jira_api_token: String,
}

impl Jira {
    pub async fn new() -> Self {
        Self { ..Jira::default() }
    }

    /// check if the user is authenticated by checking if username and the apikeys is_empty()
    /// is unauthenticated the user will prompt to authenticate
    pub async fn init(mut self) -> Result<()> {
        println!("{:?}", Secrets::get(&JiraSecrets::Username.to_string())?);
        println!(
            "{:?}",
            Secrets::get(&JiraSecrets::JiraApiToken.to_string())?
        );

        if !(self.username.is_empty() && self.jira_api_token.is_empty()) {
            self.authenticated = true;
        } else {
            helpers::promt_user("enter user name below: ")?;
            let _ = std::io::stdout().flush();
            self.username = helpers::read_stdin()?;
            helpers::promt_user("enter jira api key below: ")?;
            self.jira_api_token = helpers::read_stdin()?;

            Secrets::set(&JiraSecrets::Username.to_string(), self.username)?;
            Secrets::set(&JiraSecrets::JiraApiToken.to_string(), self.jira_api_token)?;

            self.username = Secrets::get(&JiraSecrets::Username.to_string())?;
            self.jira_api_token = Secrets::get(&JiraSecrets::JiraApiToken.to_string())?;
        }
        Ok(())
    }
}
