use colored::Colorize;
use inquire::Text;

use crate::config;
use crate::error::{KlockError, Result};
use crate::services::sessions::{
    build_session, ensure_no_active_session, first_board_for_project, perform_search_tasks,
    resolve_project,
};
use crate::session::{fuzzy_select, prompt_select, prompt_text, prompt_time};

pub async fn handle(
    project_code: Option<String>,
    search_string: Option<String>,
    start: Option<String>,
    end: Option<String>,
) -> Result<()> {
    ensure_no_active_session()?;

    let cfg = config::load_config()?;

    let code = match project_code {
        Some(c) => c,
        None => {
            if cfg.projects.is_empty() {
                return Err(KlockError::ConfigError(
                    "No projects configured. Run `klock add` to link one.".to_string(),
                ));
            }
            let options: Vec<String> = cfg
                .projects
                .iter()
                .map(|p| {
                    if p.platform_project_name.is_empty() {
                        p.code.clone()
                    } else {
                        format!("{} — {}", p.code, p.platform_project_name)
                    }
                })
                .collect();
            let picked = prompt_select("Project:", options)?;
            picked.split(" — ").next().unwrap_or(&picked).to_string()
        }
    };

    let project = resolve_project(&cfg, &code)?.clone();
    let board_id = first_board_for_project(&cfg, &project)?.id.clone();

    let query = match search_string {
        Some(s) => s,
        None => prompt_text("Search tasks:", None)?,
    };

    let tasks = perform_search_tasks(&cfg, &project, &query).await?;
    if tasks.is_empty() {
        return Err(KlockError::NotFound(format!("No tasks found for '{query}'")));
    }

    let selected = fuzzy_select(
        tasks,
        |t| format!("[{}] {} ({})", t.key, t.title, t.status),
        "Select task:",
    )?;

    let start_time = match start {
        Some(s) => Some(s),
        None => Some(prompt_time("Start time (HHMM or 'now'):", Some("now"))?),
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
    let task_key = selected.key.clone();
    let task_title = selected.title.clone();
    let session = build_session(
        &project,
        board_id,
        selected,
        start_time.clone(),
        end_time,
        active_date,
    );
    config::save_session(&session)?;

    let time_display = start_time.as_deref().unwrap_or("now");
    println!(
        "{} Started [{}] {} — {}",
        "▶".green(),
        task_key.bold(),
        task_title,
        time_display.yellow()
    );

    Ok(())
}
