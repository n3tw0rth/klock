use async_trait::async_trait;
pub mod clickup;
pub mod jira;

use crate::Args;
use crate::error::Result;

#[async_trait]
pub trait Board {
    async fn new() -> Self;
    async fn init(self, args: Args) -> Result<()>;
    async fn get_project_issues(&mut self, project_code: &String) -> Result<()>;
    async fn process_arguments(&mut self, args: Args) -> Result<()>;
    async fn logout(&self) -> Result<()>;
    async fn pick_issue(&self, issues: Vec<JiraIssue>) -> Result<String>;
    async fn fuzzy_search(
        &mut self,
        project_code: &String,
        pattern: &String,
    ) -> Result<Vec<JiraIssue>>;
}

/// These structs defines the jira issues REST API reponse
/// And will be used with serde_json
use serde::Deserialize;

#[derive(Deserialize, Debug, Default, Clone)]
struct JiraIssues {
    pub issues: Vec<JiraIssue>,
}

#[derive(Deserialize, Debug, Clone)]
struct JiraIssue {
    #[serde(rename = "self")]
    pub id: String,
    pub key: String,
    pub fields: JiraIssueFields,
}

#[derive(Deserialize, Debug, Clone)]
struct JiraIssueFields {
    summary: String,
    #[serde(rename = "statusCategory")]
    status_category: serde_json::Value,
}
