use reqwest::{Client, ClientBuilder};

use crate::common::{Secrets, helpers};
use crate::error::{Error, Result};

use super::Board;

/// Defines the types of secrets used with Jira
#[derive(Debug)]
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
    pub fn query(&self, server: &str, account_id: &str, project: &str) -> String {
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

#[derive(Default)]
pub struct Jira {
    pub authenticated: bool,
    pub server: String,
    pub username: String,
    pub jira_api_token: String,
    pub client: Client,
}

impl Board for Jira {
    /// Instantiate the Jira with the default values
    async fn new() -> Self {
        let client = ClientBuilder::new()
            .build()
            .expect("Failed to create the HTTP client");
        Self {
            client,
            ..Default::default()
        }
    }

    /// check if the user is authenticated by checking if username and the apikeys is_empty()
    /// is unauthenticated the user will prompt to authenticate
    async fn init(mut self) -> Result<()> {
        if !(Secrets::get(&JiraSecrets::JiraApiToken.to_string())?.is_empty()
            && Secrets::get(&JiraSecrets::Username.to_string())?.is_empty())
        {
            self.authenticated = true;
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

        Ok(())
    }

    async fn issues(&self) -> Result<()> {
        unimplemented!();
        //self.client
        //    .get("https://surgeglobal.atlassian.net/rest/api/3/search?{querygoeshere}")
    }
}

impl Jira {
    async fn find_account_id(self) -> Result<()> {
        let myself_url = format!("https://{}/rest/api/3/myself", self.server);
        let response = self
            .client
            .get(myself_url)
            .basic_auth(self.username, Some(self.jira_api_token))
            .send()
            .await?;
        println!("{:?}", response);
        Ok(())
    }
}
