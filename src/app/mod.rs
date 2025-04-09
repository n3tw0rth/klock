use serde::Deserialize;

pub mod jira;

pub use jira::*;

#[derive(Deserialize)]
struct Issues {
    pub issues: Issue,
}

#[derive(Deserialize)]
struct Issue {
    #[serde(rename = "self")]
    pub id: String,
    pub key: String,
    pub fields: IssueFields,
}

#[derive(Deserialize)]
struct IssueFields {
    summary: String,
    #[serde(rename = "stateCategory")]
    status_category: serde_json::Value,
}
