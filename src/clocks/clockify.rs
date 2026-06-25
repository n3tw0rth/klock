use async_trait::async_trait;
use chrono::NaiveDate;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::{Clock, TimeEntry};
use crate::error::{JiredError, Result};

pub struct ClockifyClock {
    api_key: String,
    client: Client,
}

impl ClockifyClock {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: Client::new(),
        }
    }

    async fn get_workspace_id(&self) -> Result<String> {
        let resp = self
            .client
            .get("https://api.clockify.me/api/v1/workspaces")
            .header("X-Api-Key", &self.api_key)
            .send()
            .await
            .map_err(|e| JiredError::NetworkError(e.to_string()))?;

        #[derive(Deserialize)]
        struct Workspace {
            id: String,
        }

        let workspaces: Vec<Workspace> = resp
            .json()
            .await
            .map_err(|e| JiredError::NetworkError(e.to_string()))?;

        workspaces
            .into_iter()
            .next()
            .map(|w| w.id)
            .ok_or_else(|| JiredError::PlatformError("No Clockify workspaces found".to_string()))
    }

    async fn find_project_id_by_name(
        &self,
        workspace_id: &str,
        name: &str,
    ) -> Result<Option<String>> {
        let url = format!(
            "https://api.clockify.me/api/v1/workspaces/{workspace_id}/projects?name={}&page-size=50",
            urlencode(name)
        );
        let resp = self
            .client
            .get(&url)
            .header("X-Api-Key", &self.api_key)
            .send()
            .await
            .map_err(|e| JiredError::NetworkError(e.to_string()))?;

        if !resp.status().is_success() {
            return Err(JiredError::PlatformError(format!(
                "Clockify project lookup failed {}",
                resp.status()
            )));
        }

        #[derive(Deserialize)]
        struct ClockifyProject {
            id: String,
            name: String,
        }

        let projects: Vec<ClockifyProject> = resp
            .json()
            .await
            .map_err(|e| JiredError::NetworkError(e.to_string()))?;

        Ok(projects
            .into_iter()
            .find(|p| p.name.eq_ignore_ascii_case(name))
            .map(|p| p.id))
    }

    async fn get_user_id(&self) -> Result<String> {
        let resp = self
            .client
            .get("https://api.clockify.me/api/v1/user")
            .header("X-Api-Key", &self.api_key)
            .send()
            .await
            .map_err(|e| JiredError::NetworkError(e.to_string()))?;

        #[derive(Deserialize)]
        struct User {
            id: String,
        }

        let user: User = resp
            .json()
            .await
            .map_err(|e| JiredError::NetworkError(e.to_string()))?;

        Ok(user.id)
    }
}

#[derive(Serialize)]
struct ClockifyTimeEntry {
    start: String,
    end: String,
    description: String,
    #[serde(rename = "projectId", skip_serializing_if = "Option::is_none")]
    project_id: Option<String>,
    #[serde(rename = "taskId", skip_serializing_if = "Option::is_none")]
    task_id: Option<String>,
}

fn urlencode(s: &str) -> String {
    s.bytes()
        .flat_map(|b| {
            let encoded: Vec<u8> = match b {
                b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => vec![b],
                _ => format!("%{b:02X}").into_bytes(),
            };
            encoded
        })
        .map(|b| b as char)
        .collect()
}

#[async_trait]
impl Clock for ClockifyClock {
    async fn log_time(&self, entry: &TimeEntry) -> Result<()> {
        let workspace_id = self.get_workspace_id().await?;

        let start_h = entry
            .start_time
            .as_deref()
            .and_then(|t| t.get(0..2))
            .and_then(|h| h.parse::<u32>().ok())
            .unwrap_or(9);
        let start_m = entry
            .start_time
            .as_deref()
            .and_then(|t| t.get(2..4))
            .and_then(|m| m.parse::<u32>().ok())
            .unwrap_or(0);
        let end_h = entry
            .end_time
            .as_deref()
            .and_then(|t| t.get(0..2))
            .and_then(|h| h.parse::<u32>().ok())
            .unwrap_or_else(|| start_h + (entry.hours as u32));
        let end_m = entry
            .end_time
            .as_deref()
            .and_then(|t| t.get(2..4))
            .and_then(|m| m.parse::<u32>().ok())
            .unwrap_or(start_m);

        let start = format!(
            "{}T{:02}:{:02}:00Z",
            entry.date.format("%Y-%m-%d"),
            start_h,
            start_m
        );
        let end = format!(
            "{}T{:02}:{:02}:00Z",
            entry.date.format("%Y-%m-%d"),
            end_h,
            end_m
        );

        let description = match entry.issue_key.as_deref() {
            Some(key) if !entry.description.starts_with(&format!("[{key}]")) => {
                format!("[{key}] {}", entry.description)
            }
            _ => entry.description.clone(),
        };

        let project_id = match entry.project_name.as_deref() {
            Some(name) if !name.is_empty() => {
                self.find_project_id_by_name(&workspace_id, name).await?
            }
            _ => None,
        };

        let body = ClockifyTimeEntry {
            start,
            end,
            description,
            project_id,
            task_id: None,
        };

        let url = format!(
            "https://api.clockify.me/api/v1/workspaces/{workspace_id}/time-entries"
        );
        let resp = self
            .client
            .post(&url)
            .header("X-Api-Key", &self.api_key)
            .json(&body)
            .send()
            .await
            .map_err(|e| JiredError::NetworkError(e.to_string()))?;

        if !resp.status().is_success() {
            return Err(JiredError::PlatformError(format!(
                "Clockify returned {}: {}",
                resp.status(),
                resp.text().await.unwrap_or_default()
            )));
        }
        Ok(())
    }

    async fn get_logged_time(&self, _task_id: &str, date: NaiveDate) -> Result<f32> {
        let workspace_id = self.get_workspace_id().await?;
        let user_id = self.get_user_id().await?;

        let start = format!("{}T00:00:00Z", date.format("%Y-%m-%d"));
        let end = format!("{}T23:59:59Z", date.format("%Y-%m-%d"));

        let url = format!(
            "https://api.clockify.me/api/v1/workspaces/{workspace_id}/user/{user_id}/time-entries?start={start}&end={end}"
        );

        let resp = self
            .client
            .get(&url)
            .header("X-Api-Key", &self.api_key)
            .send()
            .await
            .map_err(|e| JiredError::NetworkError(e.to_string()))?;

        if !resp.status().is_success() {
            return Err(JiredError::PlatformError(format!(
                "Clockify GET failed {}",
                resp.status()
            )));
        }

        #[derive(Deserialize)]
        struct ClockifyEntry {
            #[serde(rename = "timeInterval")]
            time_interval: TimeInterval,
        }

        #[derive(Deserialize)]
        struct TimeInterval {
            duration: Option<String>,
        }

        let entries: Vec<ClockifyEntry> = resp
            .json()
            .await
            .map_err(|e| JiredError::NetworkError(e.to_string()))?;

        let total_hours: f32 = entries
            .iter()
            .filter_map(|e| e.time_interval.duration.as_deref())
            .filter_map(parse_iso_duration)
            .sum();

        Ok(total_hours)
    }

    fn platform_name(&self) -> &str {
        "Clockify"
    }
}

fn parse_iso_duration(s: &str) -> Option<f32> {
    // Format: PT1H30M → 1.5
    let s = s.strip_prefix("PT")?;
    let hours = if let Some(h) = s.split('H').next() {
        h.parse::<f32>().unwrap_or(0.0)
    } else {
        0.0
    };
    let minutes = if s.contains('H') {
        s.split('H')
            .nth(1)
            .and_then(|m| m.strip_suffix('M'))
            .and_then(|m| m.parse::<f32>().ok())
            .unwrap_or(0.0)
    } else {
        s.strip_suffix('M')
            .and_then(|m| m.parse::<f32>().ok())
            .unwrap_or(0.0)
    };
    Some(hours + minutes / 60.0)
}
