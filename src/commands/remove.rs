use colored::Colorize;

use crate::config;
use crate::error::{JiredError, Result};
use crate::session::{prompt_confirm, prompt_select, prompt_text};

const BOARD_TAG: &str = "[board]";
const CLOCK_TAG: &str = "[clock]";

pub async fn handle(project_code: Option<String>) -> Result<()> {
    let code = match project_code {
        Some(c) => c,
        None => prompt_text("Project code:", None)?,
    };

    let mut cfg = config::load_config()?;

    let proj_idx = cfg
        .projects
        .iter()
        .position(|p| p.code.eq_ignore_ascii_case(&code))
        .ok_or_else(|| JiredError::NotFound(format!("Project '{code}' not found")))?;

    let mut options: Vec<String> = Vec::new();
    options.extend(
        cfg.projects[proj_idx]
            .board_ids
            .iter()
            .map(|id| format!("{BOARD_TAG} {id}")),
    );
    options.extend(
        cfg.projects[proj_idx]
            .clock_ids
            .iter()
            .map(|id| format!("{CLOCK_TAG} {id}")),
    );

    if options.is_empty() {
        return Err(JiredError::ConfigError(format!(
            "Project '{code}' has no integrations to remove"
        )));
    }

    let selection = prompt_select("Select integration to unlink:", options)?;

    let (kind, target_id) = if let Some(rest) = selection.strip_prefix(&format!("{BOARD_TAG} ")) {
        ("board", rest.to_string())
    } else if let Some(rest) = selection.strip_prefix(&format!("{CLOCK_TAG} ")) {
        ("clock", rest.to_string())
    } else {
        return Err(JiredError::ConfigError(format!(
            "Unrecognized selection '{selection}'"
        )));
    };

    if kind == "clock" {
        warn_about_pending_for_clock(&target_id)?;
    }

    if !prompt_confirm(&format!(
        "Unlink {kind} '{target_id}' from project '{code}'?"
    ))? {
        println!("{} Cancelled.", "–".dimmed());
        return Ok(());
    }

    let project = &mut cfg.projects[proj_idx];
    match kind {
        "board" => project.board_ids.retain(|id| id != &target_id),
        "clock" => project.clock_ids.retain(|id| id != &target_id),
        _ => unreachable!(),
    }

    config::save_config(&cfg)?;
    println!(
        "{} Unlinked {} '{}' from project '{}'",
        "✓".green(),
        kind,
        target_id.bold(),
        code.bold()
    );

    Ok(())
}

fn warn_about_pending_for_clock(clock_id: &str) -> Result<()> {
    let pending = config::load_pending()?;
    let affected: Vec<u32> = pending
        .entries
        .iter()
        .filter(|e| e.clock_ids.iter().any(|cid| cid == clock_id))
        .map(|e| e.idx)
        .collect();

    if !affected.is_empty() {
        println!(
            "{} Pending entries still target this clock: {}",
            "!".yellow(),
            affected
                .iter()
                .map(|i| format!("#{i}"))
                .collect::<Vec<_>>()
                .join(", ")
        );
        println!(
            "  Unlinking only affects future sessions. Run `jired pending push` or `jired pending remove <id>` for these first if you don't want them to push to '{clock_id}'."
        );
    }
    Ok(())
}
