use colored::Colorize;

use crate::boards::build_board;
use crate::config::{
    self,
    models::{AppConfig, BoardPlatform, ClockConfig, ClockPlatform, ProjectConfig},
};
use crate::error::{JiredError, Result};
use crate::session::{fuzzy_select, prompt_select, prompt_text};

const DERIVE_PREFIX: &str = "+ Jira worklog (from board: ";

fn derive_jira_clock_from_board(cfg: &mut AppConfig, board_id: &str) -> Result<String> {
    let board = cfg
        .boards
        .iter()
        .find(|b| b.id == board_id)
        .ok_or_else(|| JiredError::NotFound(format!("Board '{board_id}' not found")))?
        .clone();

    if let Some(existing) = cfg.clocks.iter().find(|c| {
        c.platform == ClockPlatform::Jira
            && c.base_url == board.base_url
            && c.email == board.email
    }) {
        return Ok(existing.id.clone());
    }

    let jira_clock_count = cfg
        .clocks
        .iter()
        .filter(|c| c.platform == ClockPlatform::Jira)
        .count();
    let new_id = format!("jira-worklog-{}", jira_clock_count + 1);

    let secret = config::get_credential(&format!("jired-{}", board.id), &board.email)?;
    config::store_credential(&format!("jired-{new_id}"), &board.email, &secret)?;

    cfg.clocks.push(ClockConfig {
        id: new_id.clone(),
        platform: ClockPlatform::Jira,
        base_url: board.base_url.clone(),
        email: board.email.clone(),
    });

    Ok(new_id)
}

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
        let derive_options: Vec<String> = cfg
            .projects
            .iter()
            .find(|p| p.code == code)
            .map(|proj| {
                proj.board_ids
                    .iter()
                    .filter_map(|bid| {
                        cfg.boards
                            .iter()
                            .find(|b| &b.id == bid && b.platform == BoardPlatform::Jira)
                    })
                    .filter(|board| {
                        !cfg.clocks.iter().any(|c| {
                            c.platform == ClockPlatform::Jira
                                && c.base_url == board.base_url
                                && c.email == board.email
                        })
                    })
                    .map(|b| format!("{DERIVE_PREFIX}{})", b.id))
                    .collect()
            })
            .unwrap_or_default();

        let mut options = derive_options;
        options.extend(cfg.clocks.iter().map(|c| c.id.clone()));

        if options.is_empty() {
            return Err(JiredError::ConfigError(
                "No clock integrations. Run `jired auth login` first.".to_string(),
            ));
        }

        let selection = prompt_select("Select clock integration:", options)?;

        let selected_id = if let Some(rest) = selection.strip_prefix(DERIVE_PREFIX) {
            let board_id = rest.trim_end_matches(')').to_string();
            let new_id = derive_jira_clock_from_board(&mut cfg, &board_id)?;
            println!(
                "{} Created Jira worklog clock '{}' reusing credentials from board '{}'",
                "✓".green(),
                new_id.bold(),
                board_id
            );
            new_id
        } else {
            selection
        };

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
