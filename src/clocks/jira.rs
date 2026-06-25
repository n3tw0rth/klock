use async_trait::async_trait;
use base64::{engine::general_purpose::STANDARD, Engine};
use chrono::{NaiveDate, TimeZone};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::{Clock, TimeEntry};
use crate::error::{KlockError, Result};

pub struct JiraClock {
    base_url: String,
    auth_header: String,
    client: Client,
}

impl JiraClock {
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

#[derive(Serialize)]
struct WorklogBody {
    #[serde(rename = "timeSpentSeconds")]
    time_spent_seconds: i64,
    started: String,
    comment: serde_json::Value,
}

#[derive(Deserialize)]
struct WorklogPage {
    worklogs: Vec<WorklogEntry>,
}

#[derive(Deserialize)]
struct WorklogEntry {
    #[serde(rename = "timeSpentSeconds")]
    time_spent_seconds: i64,
    started: String,
}

#[async_trait]
impl Clock for JiraClock {
    async fn log_time(&self, entry: &TimeEntry) -> Result<()> {
        let seconds = (entry.hours * 3600.0) as i64;

        let (hour, minute) = entry
            .start_time
            .as_deref()
            .and_then(parse_hhmm)
            .unwrap_or((9, 0));

        let naive = entry.date.and_hms_opt(hour, minute, 0).ok_or_else(|| {
            KlockError::PlatformError(format!("Invalid time {hour:02}{minute:02}"))
        })?;

        let started = chrono::Local
            .from_local_datetime(&naive)
            .single()
            .map(|dt| dt.format("%Y-%m-%dT%H:%M:%S.000%z").to_string())
            .ok_or_else(|| {
                KlockError::PlatformError("Could not localize start datetime".to_string())
            })?;

        let body = WorklogBody {
            time_spent_seconds: seconds,
            started,
            comment: serde_json::json!({
                "type": "doc",
                "version": 1,
                "content": [{"type": "paragraph", "content": [{"type": "text", "text": entry.description}]}]
            }),
        };

        let url = format!(
            "{}/rest/api/3/issue/{}/worklog",
            self.base_url, entry.task_id
        );
        let resp = self
            .client
            .post(&url)
            .header("Authorization", &self.auth_header)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| KlockError::NetworkError(e.to_string()))?;

        if !resp.status().is_success() {
            return Err(KlockError::PlatformError(format!(
                "Jira worklog failed {}: {}",
                resp.status(),
                resp.text().await.unwrap_or_default()
            )));
        }
        Ok(())
    }

    async fn get_logged_time(&self, task_id: &str, date: NaiveDate) -> Result<f32> {
        let url = format!("{}/rest/api/3/issue/{}/worklog", self.base_url, task_id);
        let resp = self
            .client
            .get(&url)
            .header("Authorization", &self.auth_header)
            .send()
            .await
            .map_err(|e| KlockError::NetworkError(e.to_string()))?;

        if !resp.status().is_success() {
            return Err(KlockError::PlatformError(format!(
                "Jira worklog GET failed {}",
                resp.status()
            )));
        }

        let page: WorklogPage = resp
            .json()
            .await
            .map_err(|e| KlockError::NetworkError(e.to_string()))?;

        let total_seconds: i64 = page
            .worklogs
            .iter()
            .filter(|w| w.started.starts_with(&date.format("%Y-%m-%d").to_string()))
            .map(|w| w.time_spent_seconds)
            .sum();

        Ok(total_seconds as f32 / 3600.0)
    }

    fn platform_name(&self) -> &str {
        "Jira"
    }
}

fn parse_hhmm(s: &str) -> Option<(u32, u32)> {
    if s.len() != 4 {
        return None;
    }
    let h = s.get(0..2)?.parse::<u32>().ok()?;
    let m = s.get(2..4)?.parse::<u32>().ok()?;
    if h < 24 && m < 60 {
        Some((h, m))
    } else {
        None
    }
}
