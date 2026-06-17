use colored::Colorize;

use crate::clocks::{build_clock, TimeEntry};
use crate::config;
use crate::error::{JiredError, Result};
use crate::session::prompt_time;

pub async fn handle(at: Option<String>) -> Result<()> {
    let session = config::load_session()?.ok_or_else(|| {
        JiredError::SessionError("No active session. Run `jired start` first.".to_string())
    })?;

    let stop_time = match at {
        Some(t) => t,
        None => prompt_time("Stop time (HHMM or 'now'):", Some("now"))?,
    };

    let hours = compute_hours(&session, &stop_time)?;

    let cfg = config::load_config()?;
    let project = cfg
        .projects
        .iter()
        .find(|p| p.code == session.project_code)
        .ok_or_else(|| {
            JiredError::NotFound(format!("Project '{}' not found in config", session.project_code))
        })?;

    if project.clock_ids.is_empty() {
        return Err(JiredError::ConfigError(format!(
            "Project '{}' has no clocks linked. Run `jired add` to link a clock.",
            session.project_code
        )));
    }

    for clock_id in &project.clock_ids {
        let clock_cfg = cfg
            .clocks
            .iter()
            .find(|c| &c.id == clock_id)
            .ok_or_else(|| {
                JiredError::ConfigError(format!("Clock '{clock_id}' not found in config"))
            })?;

        let clock = build_clock(clock_cfg)?;

        let entry = TimeEntry {
            task_id: session.task_id.clone(),
            hours,
            description: session.task_title.clone(),
            date: session.active_date,
            start_time: session.start_time_override.clone(),
            end_time: Some(stop_time.clone()),
        };

        clock.log_time(&entry).await.map_err(|e| {
            JiredError::NetworkError(format!("Failed to log to '{clock_id}': {e}"))
        })?;

        println!(
            "{} Logged {:.2}h to {}",
            "✓".green(),
            hours,
            clock_id.bold()
        );
    }

    config::clear_session()?;
    println!(
        "{} Session ended — {:.2}h logged for [{}]",
        "■".blue(),
        hours,
        session.task_title
    );

    Ok(())
}

fn compute_hours(session: &crate::config::models::Session, stop_time: &str) -> Result<f32> {
    let start_str = match session.start_time_override.as_deref() {
        Some(s) if !s.is_empty() => s.to_string(),
        _ => {
            // No HHMM override recorded — derive HHMM from started_at wall clock
            use chrono::Timelike;
            let t = session.started_at;
            format!("{:02}{:02}", t.hour(), t.minute())
        }
    };

    parse_hhmm_diff(&start_str, stop_time)
}

fn parse_hhmm_diff(start: &str, stop: &str) -> Result<f32> {
    let start_h: u32 = start.get(0..2).and_then(|h| h.parse().ok()).ok_or_else(|| {
        JiredError::SessionError(format!("Invalid start time '{start}'."))
    })?;
    let start_m: u32 = start.get(2..4).and_then(|m| m.parse().ok()).ok_or_else(|| {
        JiredError::SessionError(format!("Invalid start time '{start}'."))
    })?;
    let stop_h: u32 = stop.get(0..2).and_then(|h| h.parse().ok()).ok_or_else(|| {
        JiredError::SessionError(format!("Invalid stop time '{stop}'."))
    })?;
    let stop_m: u32 = stop.get(2..4).and_then(|m| m.parse().ok()).ok_or_else(|| {
        JiredError::SessionError(format!("Invalid stop time '{stop}'."))
    })?;

    let start_mins = (start_h * 60 + start_m) as i32;
    let stop_mins = (stop_h * 60 + stop_m) as i32;

    if stop_mins <= start_mins {
        return Err(JiredError::SessionError(
            "Stop time must be after start time.".to_string(),
        ));
    }

    Ok((stop_mins - start_mins) as f32 / 60.0)
}

#[cfg(test)]
mod tests {
    use super::parse_hhmm_diff;

    #[test]
    fn diff_basic() {
        assert!((parse_hhmm_diff("0900", "1100").unwrap() - 2.0).abs() < 1e-3);
    }

    #[test]
    fn diff_minutes() {
        assert!((parse_hhmm_diff("0915", "1045").unwrap() - 1.5).abs() < 1e-3);
    }

    #[test]
    fn diff_rejects_backwards() {
        assert!(parse_hhmm_diff("1100", "0900").is_err());
    }

    #[test]
    fn diff_rejects_garbage() {
        assert!(parse_hhmm_diff("ab", "1100").is_err());
    }
}
