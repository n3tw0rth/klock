use chrono::Local;
use colored::Colorize;

use crate::boards::build_board;
use crate::config::{self, models::Session};
use crate::error::{JiredError, Result};
use crate::session::{fuzzy_select, prompt_text, prompt_time};

pub async fn handle(
    project_code: Option<String>,
    search_string: Option<String>,
    start: Option<String>,
    end: Option<String>,
) -> Result<()> {
    if config::load_session()?.is_some() {
        return Err(JiredError::SessionError(
            "Session already active. Run `jired stop` first.".to_string(),
        ));
    }

    let code = match project_code {
        Some(c) => c,
        None => prompt_text("Project code:", None)?,
    };

    let cfg = config::load_config()?;
    let project = cfg
        .projects
        .iter()
        .find(|p| p.code.to_uppercase() == code.to_uppercase())
        .ok_or_else(|| {
            JiredError::NotFound(format!(
                "Project '{code}' not found. Run `jired add` to link a project."
            ))
        })?;

    if project.board_ids.is_empty() {
        return Err(JiredError::ConfigError(format!(
            "Project '{code}' has no board linked. Run `jired add` first."
        )));
    }

    let board_id = project.board_ids[0].clone();
    let board_cfg = cfg
        .boards
        .iter()
        .find(|b| b.id == board_id)
        .ok_or_else(|| JiredError::ConfigError(format!("Board '{board_id}' not found in config")))?;

    let board = build_board(board_cfg)?;

    let query = match search_string {
        Some(s) => s,
        None => prompt_text("Search tasks:", None)?,
    };

    let tasks = board
        .search_tasks(&project.platform_project_id, &query)
        .await
        .map_err(|e| JiredError::NetworkError(format!("Failed to search tasks: {e}")))?;

    if tasks.is_empty() {
        return Err(JiredError::NotFound(format!("No tasks found for '{query}'")));
    }

    let selected = fuzzy_select(
        tasks,
        |t| format!("[{}] {} ({})", t.key, t.title, t.status),
        "Select task:",
    )?;

    let start_time = match start {
        Some(s) => Some(s),
        None => {
            let t = prompt_time("Start time (HHMM or 'now'):", Some("now"))?;
            Some(t)
        }
    };

    let end_time = match end {
        Some(e) => Some(e),
        None => {
            let t = Text::new("End time (HHMM, optional — press Enter to skip):")
                .prompt()
                .unwrap_or_default();
            if t.is_empty() {
                None
            } else {
                Some(t)
            }
        }
    };

    let active_date = config::load_active_date()?;

    let session = Session {
        project_code: code.clone(),
        task_id: selected.id.clone(),
        task_key: Some(selected.key.clone()),
        task_title: selected.title.clone(),
        board_id,
        started_at: Local::now(),
        start_time_override: start_time.clone(),
        end_time_override: end_time,
        active_date,
    };

    config::save_session(&session)?;

    let time_display = start_time.as_deref().unwrap_or("now");
    println!(
        "{} Started [{}] {} — {}",
        "▶".green(),
        selected.key.bold(),
        selected.title,
        time_display.yellow()
    );

    Ok(())
}

use inquire::Text;
