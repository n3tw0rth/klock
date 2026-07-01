use colored::Colorize;

use crate::config;
use crate::error::{KlockError, Result};
use crate::services::projects::{
    pending_affected_by_clock_unlink, unlink_integration, IntegrationKind,
};
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
        .ok_or_else(|| KlockError::NotFound(format!("Project '{code}' not found")))?;

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
        return Err(KlockError::ConfigError(format!(
            "Project '{code}' has no integrations to remove"
        )));
    }

    let selection = prompt_select("Select integration to unlink:", options)?;

    let (kind, target_id) = if let Some(rest) = selection.strip_prefix(&format!("{BOARD_TAG} ")) {
        (IntegrationKind::Board, rest.to_string())
    } else if let Some(rest) = selection.strip_prefix(&format!("{CLOCK_TAG} ")) {
        (IntegrationKind::Clock, rest.to_string())
    } else {
        return Err(KlockError::ConfigError(format!(
            "Unrecognized selection '{selection}'"
        )));
    };

    if kind == IntegrationKind::Clock {
        let pending = config::load_pending()?;
        let affected = pending_affected_by_clock_unlink(&pending, &target_id);
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
                "  Unlinking only affects future sessions. Run `klock pending push` or `klock pending remove <id>` for these first if you don't want them to push to '{target_id}'."
            );
        }
    }

    if !prompt_confirm(&format!(
        "Unlink {} '{target_id}' from project '{code}'?",
        kind.label()
    ))? {
        println!("{} Cancelled.", "–".dimmed());
        return Ok(());
    }

    let kind_label = kind.label().to_string();
    unlink_integration(&mut cfg, &code, kind, &target_id)?;
    println!(
        "{} Unlinked {} '{}' from project '{}'",
        "✓".green(),
        kind_label,
        target_id.bold(),
        code.bold()
    );

    Ok(())
}
