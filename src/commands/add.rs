use colored::Colorize;

use crate::boards::build_board;
use crate::config::{self, models::ProjectConfig};
use crate::error::{JiredError, Result};
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
            return Err(JiredError::ConfigError(
                "No board integrations. Run `jired auth login` first.".to_string(),
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

        let projects = board.search_projects(&query).await.map_err(|e| {
            JiredError::NetworkError(format!("Failed to search projects: {e}"))
        })?;

        if projects.is_empty() {
            return Err(JiredError::NotFound(format!("No projects found for '{query}'")));
        }

        let selected = fuzzy_select(
            projects,
            |p| format!("[{}] {}", p.key, p.name),
            "Select project:",
        )?;

        if let Some(proj) = cfg.projects.iter_mut().find(|p| p.code == code) {
            if !proj.board_ids.contains(&selected_id) {
                proj.board_ids.push(selected_id.clone());
            }
            proj.platform_project_id = selected.id.clone();
            proj.platform_project_name = selected.name.clone();
        } else {
            cfg.projects.push(ProjectConfig {
                code: code.clone(),
                board_ids: vec![selected_id.clone()],
                clock_ids: vec![],
                platform_project_id: selected.id.clone(),
                platform_project_name: selected.name.clone(),
            });
        }

        config::save_config(&cfg)?;
        println!(
            "{} Added {} → project {}",
            "✓".green(),
            selected.name.bold(),
            code.bold()
        );
    } else {
        if cfg.clocks.is_empty() {
            return Err(JiredError::ConfigError(
                "No clock integrations. Run `jired auth login` first.".to_string(),
            ));
        }
        let clock_ids: Vec<String> = cfg.clocks.iter().map(|c| c.id.clone()).collect();
        let selected_id = prompt_select("Select clock integration:", clock_ids)?;

        if let Some(proj) = cfg.projects.iter_mut().find(|p| p.code == code) {
            if !proj.clock_ids.contains(&selected_id) {
                proj.clock_ids.push(selected_id.clone());
            }
        } else {
            cfg.projects.push(ProjectConfig {
                code: code.clone(),
                board_ids: vec![],
                clock_ids: vec![selected_id.clone()],
                platform_project_id: String::new(),
                platform_project_name: String::new(),
            });
        }

        config::save_config(&cfg)?;
        println!(
            "{} Linked clock '{}' → project {}",
            "✓".green(),
            selected_id.bold(),
            code.bold()
        );
    }

    Ok(())
}
