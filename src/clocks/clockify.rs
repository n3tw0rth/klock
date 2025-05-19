use async_trait::async_trait;
use reqwest::{Client, ClientBuilder};
use serde::{Deserialize, Serialize};
use strum::{EnumIter, IntoEnumIterator};
use tracing::info;

use super::Clock;
use crate::common::config::ConfigParser;
use crate::common::tracker::Tracker;
use crate::common::{helpers, Secrets};
use crate::error::Error;
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
        let workspace_id =
            Secrets::get(&ClockifySecrets::WorkspaceId.to_string()).unwrap_or_default();

        if !api_token.is_empty() {
            self.authenticated = true;
            self.api_token = api_token;
            self.workspace_id = workspace_id;
        } else {
            helpers::promt_user("enter the atlassian servername")?;
            self.api_token = helpers::read_stdin()?;
            Secrets::set(&ClockifySecrets::ApiToken.to_string(), &self.api_token)?;

            // get the workspace
            self.set_workspace_id().await?;
        }

        self.process_arguments(args).await?;
        Ok(())
    }

    async fn process_arguments(&mut self, args: Args) -> Result<()> {
        match args.command {
            Commands::Log => self.log().await?,
            Commands::Add { project } => {
                self.add_new_project(project).await?;
            }
            Commands::Logout => {
                self.logout().await?;
            }
            _ => {}
        }
        Ok(())
    }

    async fn logout(&self) -> Result<()> {
        for secret in ClockifySecrets::iter() {
            Secrets::delete(&secret.to_string())?;
        }
        Ok(())
    }

    async fn log(&self) -> Result<()> {
        println!("logging started");
        println!("workspaceId: {}", self.workspace_id);

        let tracker = Tracker::new().await?;
        let projects = ConfigParser::parse().await?.get_projects()?;
        let lines = tracker.read().await?;

        for line in lines.iter() {
            let parts = shell_words::split(line).map_err(|e| Error::CustomError(e.to_string()))?;

            let [key, ticket_id, end, start, title]: [String; 5] = match parts.try_into() {
                Ok(array) => array,
                Err(_) => panic!("Expected exactly 5 fields"),
            };

            let formatted_end_string = tracker.format_24_hrs(&end)?;
            let formatted_start_string = tracker.format_24_hrs(&start)?;

            let description = format!("[{}]: {}", ticket_id, title);

            let project = match projects.iter().find(|p| p.key == key) {
                Some(p) => Ok(p),
                None => Err(Error::CustomError(format!(
                    "Project with key '{}' not found",
                    key
                ))),
            }?;

            let payload = ClockifyTimeEntryPayload {
                billable: true,
                workspace_id: self.workspace_id.trim_matches('"').to_string(),
                description,
                project_id: project.id.to_string(),
                end: formatted_end_string,
                start: formatted_start_string,
            };

            // create a time entry on clockify
            let mut url = BASE_URL.to_string();
            url.push_str("/workspaces");

            let url = format!(
                "{}/workspaces/{}/time-entries",
                BASE_URL,
                self.workspace_id.trim_matches('"')
            );

            let response = self
                .client
                .post(&url)
                .header("X-Api-Key", &self.api_token)
                .json(&payload)
                .send()
                .await?
                .text()
                .await?;

            println!("{:?}", response);
        }

        Ok(())
    }
}

impl ClockifyClock {
    /// This will collect and store the clockify workspace id in the keyring
    pub async fn set_workspace_id(&mut self) -> Result<()> {
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

    /// This method is used to add a new project and save it in the config file
    pub async fn add_new_project(&self, project: String) -> Result<()> {
        let url = format!(
            "{}/workspaces/{}/projects",
            BASE_URL,
            self.workspace_id.trim_matches('"')
        );

        let response = self
            .client
            .get(url)
            .header("X-Api-Key", &self.api_token)
            .query(&[("name", &project)])
            .send()
            .await?
            .text()
            .await?;

        let json = serde_json::from_str::<Vec<ClockifyProjectsResponse>>(&response)
            .map_err(|_| ClockifyError("Error parsing projects response to json".to_string()))?;

        // select the right project
        json.iter().enumerate().for_each(|(index, item)| {
            println!("{} {:?}", index + 1, item.name);
        });

        // promt the user to select the correct code
        let selected_item = if json.len() == 1 {
            json.first()
                .ok_or("Project response is empty, check the project code again and try")
                .map_err(|e| Error::CustomError(e.to_string()))?
        } else {
            helpers::promt_user("Please select the correct project code")?;
            let user_selection = helpers::read_stdin()?;
            let index: usize = user_selection
                .trim()
                .parse()
                .expect("Please enter a valid value");
            json.get(index - 1)
                .ok_or("Project response is empty, check the project code again and try")
                .map_err(|e| Error::CustomError(e.to_string()))?
        };

        ConfigParser::parse()
            .await?
            .set_project(
                project,
                selected_item.name.clone(),
                selected_item.id.clone(),
            )?
            .update_config()
            .await?;

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

/// https://docs.developer.clockify.me/#tag/Time-entry/operation/createTimeEntry
#[derive(Deserialize, Serialize, Debug)]
pub struct ClockifyTimeEntryPayload {
    #[serde(rename = "workspaceId")]
    workspace_id: String,
    billable: bool,
    #[serde(rename = "projectId")]
    project_id: String,
    start: String,
    end: String,
    description: String,
}

#[derive(Deserialize, Debug)]
struct ClockifyProjectsResponse {
    id: String,
    name: String,
}
