pub mod clickup;
pub mod jira;

use async_trait::async_trait;

use crate::config::models::{BoardConfig, BoardPlatform};
use crate::error::Result;

#[derive(Debug, Clone)]
pub struct RemoteProject {
    pub id: String,
    pub name: String,
    pub key: String,
}

#[derive(Debug, Clone)]
pub struct RemoteTask {
    pub id: String,
    pub key: String,
    pub title: String,
    pub status: String,
    pub assignee: Option<String>,
}

#[async_trait]
pub trait Board: Send + Sync {
    async fn search_projects(&self, query: &str) -> Result<Vec<RemoteProject>>;
    async fn search_tasks(&self, project_id: &str, query: &str) -> Result<Vec<RemoteTask>>;
    fn platform_name(&self) -> &str;
}

pub fn build_board(config: &BoardConfig) -> Result<Box<dyn Board>> {
    use crate::config::{get_credential};
    let service = format!("jired-{}", config.id);
    let token = get_credential(&service, &config.email)?;
    match config.platform {
        BoardPlatform::Jira => Ok(Box::new(jira::JiraBoard::new(
            config.base_url.clone(),
            config.email.clone(),
            token,
        ))),
        BoardPlatform::ClickUp => {
            let team_id = config.team_id.clone().ok_or_else(|| {
                crate::error::JiredError::ConfigError(
                    "ClickUp board is missing team_id. Run `jired auth login` to reconfigure.".to_string(),
                )
            })?;
            Ok(Box::new(clickup::ClickUpBoard::new(token, team_id)))
        }
    }
}
