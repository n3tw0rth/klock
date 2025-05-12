use async_trait::async_trait;
use reqwest::{Client, ClientBuilder};
use serde::Deserialize;
use strum::EnumIter;
use tracing::info;

use super::Clock;
use crate::common::{helpers, Secrets};
use crate::error::{Error::ClockifyError, Result};
use crate::{Args, Commands};

const BASE_URL: &str = "https://api.clockify.me/api/v1";

#[derive(Default)]
pub struct ClockifyClock {
    api_token: String,
    authenticated: bool,
    workspace_id: String,

    client: Client,
}

#[async_trait]
impl Clock for ClockifyClock {
    async fn new() -> Self {
        let client = ClientBuilder::new()
            .build()
            .expect("Failed to create the HTTP client");
        Self {
            client,
            ..ClockifyClock::default()
        }
    }

    async fn init(&mut self, args: Args) -> Result<()> {
        let api_token = Secrets::get(&ClockifySecrets::ApiToken.to_string()).unwrap_or_default();

        if !api_token.is_empty() {
            self.authenticated = true;
            self.api_token = api_token;
        } else {
            helpers::promt_user("enter the atlassian servername")?;
            self.api_token = helpers::read_stdin()?;

            Secrets::set(&ClockifySecrets::ApiToken.to_string(), &self.api_token)?;
        }

        // get the workspace
        self.set_workspace_id().await?;
        self.process_arguments(args).await?;
        Ok(())
    }

    async fn process_arguments(&mut self, args: Args) -> Result<()> {
        match args.command {
            Commands::Log => self.log().await?,
            _ => {}
        }
        Ok(())
    }

    async fn log(&self) -> Result<()> {
        println!("logging started");
        println!("workspaceId: {}", self.workspace_id);
        Ok(())
    }
}

impl ClockifyClock {
    /// This will collect and store the clockify workspace id in the keyring
    pub async fn set_workspace_id(&mut self) -> Result<()> {
        // FIXME: if the values is already saved in the keyring do not need to make the request
        // again just update the app state
        info!("requesting the workspace id");
        let mut url = BASE_URL.to_string();
        url.push_str("/workspaces");

        let response = self
            .client
            .get(url)
            .header("X-Api-Key", &self.api_token)
            .send()
            .await?
            .text()
            .await?;

        let json = serde_json::from_str::<serde_json::Value>(&response)
            .map_err(|_| ClockifyError("Error parsing issues response to json".to_string()))?;

        let workspace_id = json
            .get(0)
            .unwrap_or(&serde_json::Value::Null)
            .get("id")
            .unwrap_or(&serde_json::Value::Null)
            .to_string();

        Secrets::set(
            &ClockifySecrets::WorkspaceId.to_string(),
            workspace_id.as_str(),
        )?;

        self.workspace_id = workspace_id;

        Ok(())
    }
}

/// Defines the types of secrets used with Clockify
#[derive(Debug, EnumIter)]
enum ClockifySecrets {
    ApiToken,
    WorkspaceId,
}

/// Implement Display trait to add .to_string() to enum fields
impl std::fmt::Display for ClockifySecrets {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}

// TODO: Temporarily commented out – planned for future use.
#[derive(Deserialize)]
pub struct ClockifyTimeEntryPayload {
    billable: bool,
    #[serde(rename = "workspaceId")]
    workspace_id: String,
}
