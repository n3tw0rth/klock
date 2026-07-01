use colored::Colorize;

use crate::boards::build_board;
use crate::config;
use crate::error::{KlockError, Result};
use crate::services::projects::{
    clock_options_for, derive_jira_clock_from_board, link_board_to_project,
    link_clock_to_project, ClockOption,
};
use crate::session::{fuzzy_select, prompt_select, prompt_text};

pub async fn handle(project_code: Option<String>, search_string: Option<String>) -> Result<()> {
    let code = match project_code {
        Some(c) => c,
        None => prompt_text("Project code:", None)?,
    };

    let kind = prompt_select(
        "Integration type:",
        vec!["Board".to_string(), "Clock".to_string()],
    )?;

    let mut cfg = config::load_config()?;

    if kind == "Board" {
        if cfg.boards.is_empty() {
            return Err(KlockError::ConfigError(
                "No board integrations. Run `klock auth login` first.".to_string(),
            ));
        }
        let board_ids: Vec<String> = cfg.boards.iter().map(|b| b.id.clone()).collect();
        let selected_id = prompt_select("Select board integration:", board_ids)?;

        let board_cfg = cfg
            .boards
            .iter()
            .find(|b| b.id == selected_id)
            .unwrap()
            .clone();
        let board = build_board(&board_cfg)?;

        let query = match search_string {
            Some(s) => s,
            None => prompt_text("Search projects:", None)?,
        };

        let projects = board
            .search_projects(&query)
            .await
            .map_err(|e| KlockError::NetworkError(format!("Failed to search projects: {e}")))?;

        if projects.is_empty() {
            return Err(KlockError::NotFound(format!("No projects found for '{query}'")));
        }

        let selected = fuzzy_select(
            projects,
            |p| format!("[{}] {}", p.key, p.name),
            "Select project:",
        )?;

        link_board_to_project(
            &mut cfg,
            &code,
            &selected_id,
            selected.id.clone(),
            selected.name.clone(),
        )?;
        println!(
            "{} Added {} → project {}",
            "✓".green(),
            selected.name.bold(),
            code.bold()
        );
    } else {
        let options = clock_options_for(&cfg, &code);
        if options.is_empty() {
            return Err(KlockError::ConfigError(
                "No clock integrations. Run `klock auth login` first.".to_string(),
            ));
        }

        let labels: Vec<String> = options
            .iter()
            .map(|opt| match opt {
                ClockOption::Existing(id) => id.clone(),
                ClockOption::Derive { board_id } => {
                    format!("+ Jira worklog (from board: {board_id})")
                }
            })
            .collect();

        let picked_label = prompt_select("Select clock integration:", labels.clone())?;
        let picked_idx = labels.iter().position(|l| l == &picked_label).unwrap();

        let selected_id = match &options[picked_idx] {
            ClockOption::Existing(id) => id.clone(),
            ClockOption::Derive { board_id } => {
                let new_id = derive_jira_clock_from_board(&mut cfg, board_id)?;
                println!(
                    "{} Created Jira worklog clock '{}' reusing credentials from board '{}'",
                    "✓".green(),
                    new_id.bold(),
                    board_id
                );
                new_id
            }
        };

        link_clock_to_project(&mut cfg, &code, &selected_id)?;
        println!(
            "{} Linked clock '{}' → project {}",
            "✓".green(),
            selected_id.bold(),
            code.bold()
        );
    }

    Ok(())
}
