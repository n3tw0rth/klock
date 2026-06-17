use async_trait::async_trait;
use base64::{engine::general_purpose::STANDARD, Engine};
use reqwest::Client;
use serde::Deserialize;

use super::{Board, RemoteProject, RemoteTask};
use crate::error::{JiredError, Result};

pub struct JiraBoard {
    base_url: String,
    auth_header: String,
    client: Client,
}

impl JiraBoard {
    pub fn new(base_url: String, email: String, api_token: String) -> Self {
        let creds = format!("{email}:{api_token}");
        let auth_header = format!("Basic {}", STANDARD.encode(creds.as_bytes()));
        Self {
            base_url,
            auth_header,
            client: Client::new(),
        }
    }
}

#[derive(Deserialize)]
struct JiraProjectPage {
    values: Vec<JiraProject>,
}

#[derive(Deserialize)]
struct JiraProject {
    id: String,
    name: String,
    key: String,
}

#[derive(Deserialize)]
struct JiraSearchResult {
    issues: Vec<JiraIssue>,
}

#[derive(Deserialize)]
struct JiraIssue {
    id: String,
    key: String,
    fields: JiraIssueFields,
}

#[derive(Deserialize)]
struct JiraIssueFields {
    summary: String,
    status: JiraStatus,
    assignee: Option<JiraAssignee>,
}

#[derive(Deserialize)]
struct JiraStatus {
    name: String,
}

#[derive(Deserialize)]
struct JiraAssignee {
    #[serde(rename = "displayName")]
    display_name: String,
}

#[async_trait]
impl Board for JiraBoard {
    async fn search_projects(&self, query: &str) -> Result<Vec<RemoteProject>> {
        let url = format!("{}/rest/api/3/project/search?query={}", self.base_url, query);
        let resp = self
            .client
            .get(&url)
            .header("Authorization", &self.auth_header)
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| JiredError::NetworkError(e.to_string()))?;

        if !resp.status().is_success() {
            return Err(JiredError::PlatformError(format!(
                "Jira returned {}: {}",
                resp.status(),
                resp.text().await.unwrap_or_default()
            )));
        }

        let page: JiraProjectPage = resp
            .json()
            .await
            .map_err(|e| JiredError::NetworkError(e.to_string()))?;

        Ok(page
            .values
            .into_iter()
            .map(|p| RemoteProject {
                id: p.id,
                name: p.name,
                key: p.key,
            })
            .collect())
    }

    async fn search_tasks(&self, project_id: &str, query: &str) -> Result<Vec<RemoteTask>> {
        let jql = format!("project={project_id}+AND+text~\"{query}\"");
        let url = format!("{}/rest/api/3/search?jql={jql}", self.base_url);
        let resp = self
            .client
            .get(&url)
            .header("Authorization", &self.auth_header)
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| JiredError::NetworkError(e.to_string()))?;

        if !resp.status().is_success() {
            return Err(JiredError::PlatformError(format!(
                "Jira returned {}: {}",
                resp.status(),
                resp.text().await.unwrap_or_default()
            )));
        }

        let result: JiraSearchResult = resp
            .json()
            .await
            .map_err(|e| JiredError::NetworkError(e.to_string()))?;

        Ok(result
            .issues
            .into_iter()
            .map(|i| RemoteTask {
                id: i.id,
                key: i.key,
                title: i.fields.summary,
                status: i.fields.status.name,
                assignee: i.fields.assignee.map(|a| a.display_name),
            })
            .collect())
    }

    fn platform_name(&self) -> &str {
        "Jira"
    }
}
