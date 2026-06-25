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
    #[serde(default)]
    pub task_key: Option<String>,
    pub task_title: String,
    pub board_id: String,
    pub started_at: DateTime<Local>,
    pub start_time_override: Option<String>,
    pub end_time_override: Option<String>,
    pub active_date: NaiveDate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingEntry {
    pub idx: u32,
    #[serde(default)]
    pub clock_ids: Vec<String>,
    #[serde(default)]
    pub pushed_clock_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clock_id: Option<String>,
    pub project_code: String,
    pub platform_project_name: String,
    pub task_id: String,
    pub task_key: Option<String>,
    pub task_title: String,
    pub description: String,
    pub date: NaiveDate,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub hours: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PendingStore {
    #[serde(default)]
    pub next_idx: u32,
    #[serde(default)]
    pub entries: Vec<PendingEntry>,
}
