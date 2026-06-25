pub mod clockify;
pub mod jira;

use async_trait::async_trait;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::config::models::{ClockConfig, ClockPlatform};
use crate::error::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeEntry {
    pub task_id: String,
    #[serde(default)]
    pub issue_key: Option<String>,
    #[serde(default)]
    pub project_name: Option<String>,
    pub hours: f32,
    pub description: String,
    pub date: NaiveDate,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
}

#[async_trait]
pub trait Clock: Send + Sync {
    async fn log_time(&self, entry: &TimeEntry) -> Result<()>;
    async fn get_logged_time(&self, task_id: &str, date: NaiveDate) -> Result<f32>;
    fn platform_name(&self) -> &str;
}

pub fn build_clock(config: &ClockConfig) -> Result<Box<dyn Clock>> {
    use crate::config::get_credential;
    let service = format!("klock-{}", config.id);
    let token = get_credential(&service, &config.email)?;
    match config.platform {
        ClockPlatform::Jira => Ok(Box::new(jira::JiraClock::new(
            config.base_url.clone(),
            config.email.clone(),
            token,
        ))),
        ClockPlatform::Clockify => Ok(Box::new(clockify::ClockifyClock::new(token))),
    }
}
