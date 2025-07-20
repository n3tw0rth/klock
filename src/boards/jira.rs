use async_trait::async_trait;
use clap::ValueEnum;
use regex::Regex;
use reqwest::{Client, ClientBuilder};
use strum::{EnumIter, IntoEnumIterator};
use tracing::info;

use crate::boards::{JiraIssue, JiraIssues};
use crate::common::{helpers, tracker::Tracker, Secrets};
use crate::error::{Error, Result};
use crate::{Args, Commands, ProjectType, StartSubcommandA, StartSubcommandB};

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
}

/// Implements a query() method on each enum field to return a query string
impl JiraSearchQuery {
    /// returns the query string for each enum field
    pub fn query(&self, server: &str, account_id: &str, project: &str) -> String {
        let base_url = format!("https://{}/rest/api/3/search", server);

        let jql_raw = match self {
            JiraSearchQuery::IssuesOnProjectQuery => {
                format!(
                    "assignee={} and project={}",
                    account_id,
                    project.to_uppercase()
                )
            }
        };

        let jql_encoded = urlencoding::encode(&jql_raw);

        format!(
            "{base_url}?jql={}&fields=key,summary,statusCategory&maxResults=50",
            jql_encoded
        )
    }
}

#[derive(Default, Debug)]
pub struct Jira {
    issues: JiraIssues,

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
        let tracker = Tracker::new().await.expect("failed to create the tracker");
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
                // check if there is a ongoing task
                // if there is one first stop that i create the new entry
                // assumming users do no work on multiple tasks at once
                //
                // TODO: it is better to introduce a functionality to work on multiple tasks at the
                // same time while acknowledging the user that there are muliplt tasks ongoing and
                // do they want to stop those tasks.
                // (eg: when starting a new task CLI will show a list of ongoing tasks. and users
                // can stop individual task using the stop subcommand passing the index of the item
                // in the list)

                let search_result = self.fuzzy_search(&project_code, &pattern).await?;

                if search_result.is_empty() {
                    return Err(Error::CustomError(
                        "There are no tickets matching your search".to_string(),
                    ));
                }

                let (task_id, task_summary) = if search_result.len() != 1 {
                    self.pick_issue(search_result).await?
                } else {
                    match search_result.first() {
                        Some(x) => (x.key.clone(), x.fields.summary.clone()),
                        None => (String::new(), String::new()),
                    }
                };

                let start_and_end_slice = match till.unwrap_or_default() {
                    StartSubcommandA::Till { till, from } => (till, from.unwrap_or_default()),
                };

                let end_time = match start_and_end_slice.clone().1 {
                    StartSubcommandB::From { start } => start,
                };

                // stop any ongoing task, assuming the user stopped the the last task at current time
                self.tracker.stop_current("-1".to_string()).await?;

                // Write the record into the file
                self.tracker
                    .create_entry(
                        &project_code,
                        &task_id,
                        start_and_end_slice.0,
                        end_time,
                        task_summary,
                    )
                    .await?
            }
            Commands::Logout => {
                self.logout().await?;
            }
            Commands::Stop { at } => {
                let end_time = match at {
                    Some(at) => at,
                    None => String::from("-1"),
                };

                self.tracker.stop_current(end_time).await?
            }
            Commands::Set { date } => {
                let re = Regex::new(r"^\d{4}-\d{2}-\d{2}$")
                    .map_err(|e| Error::CustomError(e.to_string()))?;

                if !re.is_match(&date) {
                    return Err(Error::CustomError(
                        "invalid time format. Support only YYYY-MM-DD".to_string(),
                    ));
                } else {
                    // Get the current operating system
                    let os = std::env::consts::OS;

                    // Print the appropriate command based on the OS
                    match os {
                        "linux" | "macos" => {
                            println!("export JIRED_CURRENT_TIME={}", date);
                        }
                        "windows" => {
                            println!("set JIRED_CURRENT_TIME={}", date);
                        }
                        _ => {
                            Error::CustomError(format!("Unsupported OS {os}"));
                        }
                    }
                }
            }
            Commands::Edit => self.tracker.open_editor().await,
            Commands::Add {
                project_type,
                key,
                pattern,
            } => self.add(project_type, &key, &pattern).await?,
            _ => {}
        }

        Ok(())
    }

    /// When there are multiple matches from a fuzzy search, this method will let the user select
    /// the right issue
    async fn pick_issue(&self, issues: Vec<JiraIssue>) -> Result<(String, String)> {
        println!("Please select an issue from the list below:\n");
        issues.iter().enumerate().for_each(|(index, issue)| {
            println!("{}. {} {}", index + 1, issue.key, issue.fields.summary);
        });

        helpers::promt_user("please select the correct issues and enter the number here: ")?;
        let input: usize = helpers::read_stdin()?
            .trim()
            .parse()
            .map_err(|_| Error::CustomError("Please enter a valid number".to_string()))?;

        let selection = issues
            .get(input - 1)
            .map(|v| (v.key.clone(), v.fields.summary.clone()))
            .unwrap_or_default();

        println!("you selected :{selection:?}");

        Ok(selection.clone())
    }

    /// check if the user is authenticated by checking if username and the apikeys is_empty()
    /// is unauthenticated the user will prompt to authenticate
    async fn init(mut self, args: Args) -> Result<()> {
        info!("jira init");
        let jira_api_token =
            Secrets::get(&JiraSecrets::JiraApiToken.to_string()).unwrap_or_default();
        let username = Secrets::get(&JiraSecrets::Username.to_string()).unwrap_or_default();

        if !jira_api_token.is_empty() && !username.is_empty() {
            self.authenticated = true;

            self.username = Secrets::get(&JiraSecrets::Username.to_string())?;
            self.server = Secrets::get(&JiraSecrets::Server.to_string())?;
            self.jira_api_token = Secrets::get(&JiraSecrets::JiraApiToken.to_string())?;
        } else {
            println!();
            println!("==================================================");
            println!("You are NOT authenticated with Jira.");
            println!("Please provide the following credentials to proceed");
            println!("- For the servername visit on of your project's board and copy. Omit https:// just paste the server name in the pattern xxx.atlassian.net");
            println!(
                "- For the user name enter the email provided by the company (used with jira)"
            );
            println!("- You can generate a jira api token here: https://id.atlassian.com/manage-profile/security/api-tokens");
            println!("Please provide the following credentials to proceed");
            println!("==================================================");
            helpers::promt_user("Enter the atlassian servername (xxx.atlassian.net)")?;
            self.server = helpers::read_stdin()?;
            helpers::promt_user("Enter user name below: ")?;
            self.username = helpers::read_stdin()?;
            helpers::promt_user("Enter jira api key below: ")?;
            self.jira_api_token = helpers::read_stdin()?;

            Secrets::set(&JiraSecrets::Username.to_string(), &self.username)?;
            Secrets::set(&JiraSecrets::JiraApiToken.to_string(), &self.jira_api_token)?;
            Secrets::set(&JiraSecrets::Server.to_string(), &self.server)?;
        }

        // TODO: prevent finding the account when user trying to logout
        // self.find_account_id().await?;
        self.process_arguments(args).await?;

        Ok(())
    }

    /// Collects issues on demand
    async fn get_project_issues(&mut self, project_code: &str) -> Result<()> {
        info!("getting issues from projects");
        self.find_account_id().await?;

        let query = JiraSearchQuery::IssuesOnProjectQuery.query(
            &self.server,
            &self.account_id,
            project_code,
        );

        let response = self
            .client
            .get(query)
            .basic_auth(&self.username, Some(&self.jira_api_token))
            .send()
            .await?
            .text()
            .await?;

        println!("{response}");

        let json: JiraIssues = serde_json::from_str(&response)
            .map_err(|_| Error::CustomError("Error parsing issues response to json".to_string()))?;

        self.issues = json;
        Ok(())
    }

    async fn logout(&self) -> Result<()> {
        info!("logging out, removing jira credentials");
        for secret in JiraSecrets::iter() {
            Secrets::delete(&secret.to_string())?;
        }
        Ok(())
    }

    /// Search thru all the issues under the specific project and return all the issues match the
    /// pattern
    async fn fuzzy_search(&mut self, project_code: &str, pattern: &str) -> Result<Vec<JiraIssue>> {
        self.get_project_issues(project_code).await?;
        info!("searching for pattern {pattern:?}");
        let filtered_issues = self
            .issues
            .issues
            .iter()
            .filter(|issue| {
                let is_inprogess = issue
                    .fields
                    .status_category
                    .get("name")
                    .unwrap_or(&serde_json::Value::default())
                    .eq("In Progress");
                issue.fields.summary.to_lowercase().contains(pattern) && is_inprogess
            })
            .cloned()
            .collect::<Vec<JiraIssue>>();

        Ok(filtered_issues)
    }

    //WIP: adding jira projects
    async fn add(&self, project_code: ProjectType, _key: &str, _pattern: &str) -> Result<()> {
        if project_code
            .to_possible_value()
            .unwrap_or_default()
            .get_name()
            .eq("jira")
        {
            //WIP: saving jira projects
        }
        Ok(())
    }
}

impl Jira {
    async fn find_account_id(&mut self) -> Result<()> {
        info!("finding user jira account id");
        if !self.account_id.is_empty() {
            info!("skipping finding user jira account id, already existing");
            return Ok(());
        }

        let myself_url = format!("https://{}/rest/api/3/myself", self.server);

        let response = self
            .client
            .get(myself_url.clone())
            .basic_auth(&self.username, Some(&self.jira_api_token))
            .send()
            .await?;

        if response.status().is_success() {
            let body = response.text().await?;
            let json: serde_json::Value = serde_json::from_str(&body)
                .map_err(|_| Error::CustomError("Failed to get the jira account id".to_string()))?;

            self.account_id = json
                .get("accountId")
                .ok_or_else(|| {
                    Error::CustomError("Missing or invalid 'accountId' field".to_string())
                })?
                .to_string();
            Ok(())
        } else {
            let status = response.status();
            // update the jira token when the token is expired
            // FIXME: better to check specifically for 401 error instead of looking for 4xx status
            // codes
            if status.is_client_error() {
                helpers::promt_user("enter jira api key below: ")?;
                self.jira_api_token = helpers::read_stdin()?;
                Secrets::set(&JiraSecrets::JiraApiToken.to_string(), &self.jira_api_token)?;
            }
            Err(Error::CustomError(
                "Your token was expired, please try again".to_string(),
            ))
        }?;
        Ok(())
    }
}
