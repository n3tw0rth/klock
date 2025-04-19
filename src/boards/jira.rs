use async_trait::async_trait;
use reqwest::{Client, ClientBuilder};
use strum::{EnumIter, IntoEnumIterator};
use tracing::info;

use crate::common::{Secrets, helpers, tracker::Tracker};
use crate::error::{Error, Result};
use crate::{Args, Commands, StartSubcommandA, StartSubcommandB};

use super::Board;

/// Defines the types of secrets used with Jira
#[derive(Debug, EnumIter)]
enum JiraSecrets {
    Username,
    JiraApiToken,
    AccountId,
    Server,
}

/// Implement Display trait to add .to_string() to enum fields
impl std::fmt::Display for JiraSecrets {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}

/// JQL query slices for different scenarios
enum JiraSearchQuery {
    /// All Issues assigned to the user in a specific project
    IssuesOnProjectQuery,
    BlankQuery,
}

/// Implements a query() method on each enum field to return a query string
impl JiraSearchQuery {
    /// returns the query string for each enum field
    pub fn query(&self, server: &str, account_id: &str, project: &String) -> String {
        let search_url = format!("https://{}/rest/api/3/search?", server);
        let query_slice = match self {
            JiraSearchQuery::IssuesOnProjectQuery => {
                format!(
                    "jql=assignee%3D{}%20and%20project%3D{}%20&fields=key,summary,statusCategory&maxResults=",
                    account_id,
                    project.to_uppercase()
                )
            }
            JiraSearchQuery::BlankQuery => "".to_owned(),
        };
        format!("{search_url}{query_slice}")
    }
}

#[derive(Default, Debug)]
pub struct Jira {
    pub authenticated: bool,
    pub server: String,
    pub username: String,
    pub jira_api_token: String,
    pub client: Client,
    pub account_id: String,

    pub tracker: Tracker,
}

#[async_trait]
impl Board for Jira {
    /// Instantiate the Jira with the default values
    async fn new() -> Self {
        let client = ClientBuilder::new()
            .build()
            .expect("Failed to create the HTTP client");
        let tracker = Tracker::new().await;
        Self {
            client,
            tracker,
            ..Default::default()
        }
    }

    async fn process_arguments(&mut self, args: Args) -> Result<()> {
        info!("Processing Arguments");
        match args.command {
            Commands::Start {
                project_code,
                pattern,
                till,
            } => {
                let search_result = self.fuzzy_search(&project_code, &pattern).await?;
                let start_and_end_slice = match till.unwrap_or_default() {
                    StartSubcommandA::Till { till, from } => (till, from.unwrap_or_default()),
                };

                let end_time = match start_and_end_slice.clone().1 {
                    StartSubcommandB::From { start } => start,
                };

                self.tracker
                    .create_entry(&project_code, &pattern, start_and_end_slice.0, end_time)
                    .await?
            }
            Commands::Logout {} => {
                self.logout().await?;
            }
            Commands::Stop { at } => {
                println!("stoping  at {:?}", at);
            }
        }

        Ok(())
    }

    /// check if the user is authenticated by checking if username and the apikeys is_empty()
    /// is unauthenticated the user will prompt to authenticate
    async fn init(mut self, args: Args) -> Result<()> {
        if !(Secrets::get(&JiraSecrets::JiraApiToken.to_string())?.is_empty()
            && Secrets::get(&JiraSecrets::Username.to_string())?.is_empty())
        {
            self.authenticated = true;

            self.username = Secrets::get(&JiraSecrets::Username.to_string())?;
            self.server = Secrets::get(&JiraSecrets::Server.to_string())?;
            self.jira_api_token = Secrets::get(&JiraSecrets::JiraApiToken.to_string())?;
        } else {
            helpers::promt_user("enter the atlassian servername")?;
            self.server = helpers::read_stdin()?;
            helpers::promt_user("enter user name below: ")?;
            self.username = helpers::read_stdin()?;
            helpers::promt_user("enter jira api key below: ")?;
            self.jira_api_token = helpers::read_stdin()?;

            Secrets::set(&JiraSecrets::Username.to_string(), &self.username)?;
            Secrets::set(&JiraSecrets::JiraApiToken.to_string(), &self.jira_api_token)?;
            Secrets::set(&JiraSecrets::Server.to_string(), &self.server)?;

            self.find_account_id().await?;
        }

        self.process_arguments(args).await?;

        Ok(())
    }

    /// Collects issues on demand
    async fn get_project_issues(&self, project_code: &String) -> Result<()> {
        let query = JiraSearchQuery::IssuesOnProjectQuery.query(
            &self.server,
            &self.account_id,
            &project_code,
        );

        let response = self
            .client
            .get(query)
            .basic_auth(self.username.clone(), Some(self.jira_api_token.clone()))
            .send()
            .await?;
        println!("{response:?}");
        Ok(())
    }

    async fn logout(&self) -> Result<()> {
        for secret in JiraSecrets::iter() {
            Secrets::delete(&secret.to_string()).map_err(|e| e)?
        }
        Ok(())
    }

    /// TODO: Currently working on this
    async fn fuzzy_search(&mut self, project_code: &String, _pattern: &String) -> Result<()> {
        self.get_project_issues(project_code).await?;
        Ok(())
    }
}

impl Jira {
    async fn find_account_id(&mut self) -> Result<()> {
        let myself_url = format!("https://{}/rest/api/3/myself", self.server);
        let response = self
            .client
            .get(myself_url)
            .basic_auth(&self.username, Some(&self.jira_api_token))
            .send()
            .await?
            .text()
            .await?;
        let json: serde_json::Value = serde_json::from_str(&response)
            .map_err(|_| Error::CustomError("Error parsing json".to_string()))?;

        self.account_id = json
            .get("accountId")
            .ok_or_else(|| Error::CustomError("Missing or invalid 'accountId' field".to_string()))?
            .to_string();
        println!(
            "test: {:?}",
            json.get("accountId").unwrap_or(&serde_json::Value::Null)
        );
        Ok(())
    }
}
