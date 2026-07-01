use chrono::{Local, NaiveDate};

use crate::boards::{build_board, RemoteTask};
use crate::config::{
    self,
    models::{AppConfig, BoardConfig, PendingEntry, ProjectConfig, Session},
};
use crate::error::{KlockError, Result};
use crate::utils::time::hhmm_diff_hours;

pub fn ensure_no_active_session() -> Result<()> {
    if config::load_session()?.is_some() {
        return Err(KlockError::SessionError(
            "Session already active. Stop it first.".to_string(),
        ));
    }
    Ok(())
}

pub fn resolve_project<'a>(cfg: &'a AppConfig, code: &str) -> Result<&'a ProjectConfig> {
    cfg.projects
        .iter()
        .find(|p| p.code.eq_ignore_ascii_case(code))
        .ok_or_else(|| {
            KlockError::NotFound(format!(
                "Project '{code}' not found. Link one with `klock add`."
            ))
        })
}

pub fn first_board_for_project<'a>(
    cfg: &'a AppConfig,
    project: &ProjectConfig,
) -> Result<&'a BoardConfig> {
    let board_id = project.board_ids.first().ok_or_else(|| {
        KlockError::ConfigError(format!(
            "Project '{}' has no board linked. Run `klock add` first.",
            project.code
        ))
    })?;
    cfg.boards
        .iter()
        .find(|b| &b.id == board_id)
        .ok_or_else(|| KlockError::ConfigError(format!("Board '{board_id}' not found in config")))
}

pub async fn perform_search_tasks(
    cfg: &AppConfig,
    project: &ProjectConfig,
    query: &str,
) -> Result<Vec<RemoteTask>> {
    let board_cfg = first_board_for_project(cfg, project)?;
    let board = build_board(board_cfg)?;
    board
        .search_tasks(&project.platform_project_id, query)
        .await
        .map_err(|e| KlockError::NetworkError(format!("Failed to search tasks: {e}")))
}

pub fn build_session(
    project: &ProjectConfig,
    board_id: String,
    task: RemoteTask,
    start: Option<String>,
    end: Option<String>,
    active_date: NaiveDate,
) -> Session {
    Session {
        project_code: project.code.clone(),
        task_id: task.id,
        task_key: Some(task.key),
        task_title: task.title,
        board_id,
        started_at: Local::now(),
        start_time_override: start,
        end_time_override: end,
        active_date,
    }
}

pub fn build_pending_entry(
    session: &Session,
    project: &ProjectConfig,
    stop_hhmm: &str,
) -> Result<PendingEntry> {
    let start_str = match session.start_time_override.as_deref() {
        Some(s) if !s.is_empty() => s.to_string(),
        _ => {
            use chrono::Timelike;
            let t = session.started_at;
            format!("{:02}{:02}", t.hour(), t.minute())
        }
    };

    let hours = hhmm_diff_hours(&start_str, stop_hhmm)?;

    if project.clock_ids.is_empty() {
        return Err(KlockError::ConfigError(format!(
            "Project '{}' has no clocks linked. Run `klock add` to link a clock.",
            session.project_code
        )));
    }

    Ok(PendingEntry {
        idx: 0,
        clock_ids: project.clock_ids.clone(),
        pushed_clock_ids: Vec::new(),
        clock_id: None,
        project_code: project.code.clone(),
        platform_project_name: project.platform_project_name.clone(),
        task_id: session.task_id.clone(),
        task_key: session.task_key.clone(),
        task_title: session.task_title.clone(),
        description: session.task_title.clone(),
        date: session.active_date,
        start_time: session.start_time_override.clone(),
        end_time: Some(stop_hhmm.to_string()),
        hours,
    })
}

pub fn set_active_date(date: NaiveDate) -> Result<()> {
    if let Some(mut session) = config::load_session()? {
        session.active_date = date;
        config::save_session(&session)?;
    }
    config::save_active_date(date)
}
