use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

use super::{Board, RemoteProject, RemoteTask};
use crate::error::{KlockError, Result};

pub struct ClickUpBoard {
    api_token: String,
    team_id: String,
    client: Client,
}

impl ClickUpBoard {
    pub fn new(api_token: String, team_id: String) -> Self {
        Self {
            api_token,
            team_id,
            client: Client::new(),
        }
    }
}

#[derive(Deserialize)]
struct SpacesResponse {
    spaces: Vec<ClickUpSpace>,
}

#[derive(Deserialize)]
struct ClickUpSpace {
    id: String,
    name: String,
}

#[derive(Deserialize)]
struct TasksResponse {
    tasks: Vec<ClickUpTask>,
}

#[derive(Deserialize)]
struct ClickUpTask {
    id: String,
    name: String,
    status: ClickUpStatus,
    assignees: Vec<ClickUpAssignee>,
}

#[derive(Deserialize)]
struct ClickUpStatus {
    status: String,
}

#[derive(Deserialize)]
struct ClickUpAssignee {
    username: String,
}

#[async_trait]
impl Board for ClickUpBoard {
    async fn search_projects(&self, query: &str) -> Result<Vec<RemoteProject>> {
        let url = format!(
            "https://api.clickup.com/api/v2/team/{}/space",
            self.team_id
        );
        let resp = self
            .client
            .get(&url)
            .header("Authorization", &self.api_token)
            .send()
            .await
            .map_err(|e| KlockError::NetworkError(e.to_string()))?;

        if !resp.status().is_success() {
            return Err(KlockError::PlatformError(format!(
                "ClickUp returned {}: {}",
                resp.status(),
                resp.text().await.unwrap_or_default()
            )));
        }

        let result: SpacesResponse = resp
            .json()
            .await
            .map_err(|e| KlockError::NetworkError(e.to_string()))?;

        let q = query.to_lowercase();
        Ok(result
            .spaces
            .into_iter()
            .filter(|s| q.is_empty() || s.name.to_lowercase().contains(&q))
            .map(|s| RemoteProject {
                id: s.id.clone(),
                key: s.id,
                name: s.name,
            })
            .collect())
    }

    async fn search_tasks(&self, _project_id: &str, query: &str) -> Result<Vec<RemoteTask>> {
        let url = format!(
            "https://api.clickup.com/api/v2/team/{}/task?query={}",
            self.team_id, query
        );
        let resp = self
            .client
            .get(&url)
            .header("Authorization", &self.api_token)
            .send()
            .await
            .map_err(|e| KlockError::NetworkError(e.to_string()))?;

        if !resp.status().is_success() {
            return Err(KlockError::PlatformError(format!(
                "ClickUp returned {}: {}",
                resp.status(),
                resp.text().await.unwrap_or_default()
            )));
        }

        let result: TasksResponse = resp
            .json()
            .await
            .map_err(|e| KlockError::NetworkError(e.to_string()))?;

        Ok(result
            .tasks
            .into_iter()
            .map(|t| RemoteTask {
                id: t.id.clone(),
                key: t.id,
                title: t.name,
                status: t.status.status,
                assignee: t.assignees.into_iter().next().map(|a| a.username),
            })
            .collect())
    }

    fn platform_name(&self) -> &str {
        "ClickUp"
    }
}
