use chrono::{DateTime, Local, NaiveDate};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum BoardPlatform {
    Jira,
    ClickUp,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ClockPlatform {
    Jira,
    Clockify,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoardConfig {
    pub id: String,
    pub platform: BoardPlatform,
    pub base_url: String,
    pub email: String,
    #[serde(default)]
    pub team_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClockConfig {
    pub id: String,
    pub platform: ClockPlatform,
    pub base_url: String,
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub code: String,
    pub board_ids: Vec<String>,
    pub clock_ids: Vec<String>,
    pub platform_project_id: String,
    pub platform_project_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    #[serde(default)]
    pub boards: Vec<BoardConfig>,
    #[serde(default)]
    pub clocks: Vec<ClockConfig>,
    #[serde(default)]
    pub projects: Vec<ProjectConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub project_code: String,
    pub task_id: String,
    pub task_title: String,
    pub board_id: String,
    pub started_at: DateTime<Local>,
    pub start_time_override: Option<String>,
    pub end_time_override: Option<String>,
    pub active_date: NaiveDate,
}
