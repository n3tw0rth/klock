use async_trait::async_trait;
use reqwest::{Client, ClientBuilder};
use strum::EnumIter;
use tracing::info;

use super::Clock;
use crate::common::{Secrets, helpers};
use crate::error::{Error::ClockifyError, Result};

const BASE_URL: &str = "https://api.clockify.me/api/v1";

#[derive(Default)]
pub struct ClockifyClock {
    api_token: String,
    authenticated: bool,

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

    async fn init(&mut self) -> Result<()> {
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
        Ok(())
    }

    async fn log() -> Result<()> {
        unimplemented!()
    }
}

impl ClockifyClock {
    /// This will collect and store the clockify workspace id in the keyring
    pub async fn set_workspace_id(&self) -> Result<()> {
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
            .unwrap_or(&serde_json::Value::Null);

        Secrets::set(
            &ClockifySecrets::WorkspaceId.to_string(),
            &workspace_id.to_string(),
        )?;

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
//#[derive(Deserialize, Debug, Clone)]
//pub struct ClockifyTimeEntryPayload {
//    billable: bool,
//    #[serde(rename = "workspaceId")]
//    workspace_id: String,
//}
