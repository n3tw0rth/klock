use colored::Colorize;

use crate::config;
use crate::error::{KlockError, Result};
use crate::services::sessions::build_pending_entry;
use crate::session::prompt_time;

pub async fn handle(at: Option<String>) -> Result<()> {
    let session = config::load_session()?.ok_or_else(|| {
        KlockError::SessionError("No active session. Run `klock start` first.".to_string())
    })?;

    let stop_time = match at {
        Some(t) => t,
        None => prompt_time("Stop time (HHMM or 'now'):", Some("now"))?,
    };

    let cfg = config::load_config()?;
    let project = cfg
        .projects
        .iter()
        .find(|p| p.code == session.project_code)
        .ok_or_else(|| {
            KlockError::NotFound(format!("Project '{}' not found in config", session.project_code))
        })?;

    for clock_id in &project.clock_ids {
        if !cfg.clocks.iter().any(|c| &c.id == clock_id) {
            return Err(KlockError::ConfigError(format!(
                "Clock '{clock_id}' not found in config"
            )));
        }
    }

    let entry = build_pending_entry(&session, project, &stop_time)?;
    let hours = entry.hours;
    let clocks_summary = project.clock_ids.join(", ");
    let task_title = session.task_title.clone();

    let idx = config::append_pending(entry)?;
    config::clear_session()?;

    println!(
        "{} Queued #{} — {:.2}h for [{}] → {} (run `klock pending push`)",
        "■".blue(),
        idx,
        hours,
        task_title,
        clocks_summary.bold()
    );

    Ok(())
}
