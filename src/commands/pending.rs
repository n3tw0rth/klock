use colored::Colorize;

use crate::cli::PendingAction;
use crate::clocks::{build_clock, TimeEntry};
use crate::config;
use crate::error::{JiredError, Result};
use crate::session::prompt_confirm;

pub async fn handle(action: Option<PendingAction>) -> Result<()> {
    match action.unwrap_or(PendingAction::List) {
        PendingAction::List => list(),
        PendingAction::Edit {
            idx,
            hours,
            start,
            end,
            description,
        } => edit(idx, hours, start, end, description),
        PendingAction::Remove { idx } => remove(idx),
        PendingAction::Push { yes } => push(yes).await,
    }
}

fn format_clocks(entry: &config::models::PendingEntry) -> String {
    entry
        .clock_ids
        .iter()
        .map(|cid| {
            if entry.pushed_clock_ids.contains(cid) {
                format!("{cid} ✓")
            } else {
                cid.clone()
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn list() -> Result<()> {
    let store = config::load_pending()?;
    if store.entries.is_empty() {
        println!("{} No pending entries.", "–".dimmed());
        return Ok(());
    }
    println!("{}", "Pending entries:".bold());
    println!("{}", "─".repeat(72));
    for e in &store.entries {
        let key = e.task_key.as_deref().unwrap_or(&e.task_id);
        println!(
            "#{:<3} [{}] {} ({}h, {})",
            e.idx,
            key.bold(),
            e.description,
            format!("{:.2}", e.hours).yellow(),
            e.date
        );
        println!(
            "      project: {} | {}→{}",
            e.platform_project_name,
            e.start_time.as_deref().unwrap_or("?"),
            e.end_time.as_deref().unwrap_or("?")
        );
        println!("      clocks:  {}", format_clocks(e));
    }
    println!("{}", "─".repeat(72));
    Ok(())
}

fn edit(
    idx: u32,
    hours: Option<f32>,
    start: Option<String>,
    end: Option<String>,
    description: Option<String>,
) -> Result<()> {
    let mut store = config::load_pending()?;
    let entry = store
        .entries
        .iter_mut()
        .find(|e| e.idx == idx)
        .ok_or_else(|| JiredError::NotFound(format!("Pending entry #{idx} not found")))?;

    if !entry.pushed_clock_ids.is_empty() {
        return Err(JiredError::ConfigError(format!(
            "Pending #{idx} has already been partially pushed to {}. Remove it or finish the push instead of editing.",
            entry.pushed_clock_ids.join(", ")
        )));
    }

    if let Some(h) = hours {
        entry.hours = h;
    }
    if let Some(s) = start {
        entry.start_time = Some(s);
    }
    if let Some(e) = end {
        entry.end_time = Some(e);
    }
    if let Some(d) = description {
        entry.description = d;
    }

    config::save_pending(&store)?;
    println!("{} Updated pending #{}", "✓".green(), idx);
    Ok(())
}

fn remove(idx: u32) -> Result<()> {
    let mut store = config::load_pending()?;
    let before = store.entries.len();
    store.entries.retain(|e| e.idx != idx);
    if store.entries.len() == before {
        return Err(JiredError::NotFound(format!(
            "Pending entry #{idx} not found"
        )));
    }
    config::save_pending(&store)?;
    println!("{} Removed pending #{}", "✓".green(), idx);
    Ok(())
}

async fn push(skip_confirm: bool) -> Result<()> {
    let store = config::load_pending()?;
    if store.entries.is_empty() {
        println!("{} No pending entries to push.", "–".dimmed());
        return Ok(());
    }

    let cfg = config::load_config()?;

    println!(
        "About to push {} pending entr{}.",
        store.entries.len(),
        if store.entries.len() == 1 { "y" } else { "ies" }
    );
    if !skip_confirm && !prompt_confirm("Proceed?")? {
        println!("{} Cancelled.", "–".dimmed());
        return Ok(());
    }

    let next_idx = store.next_idx;
    let mut remaining = Vec::new();
    let mut fully_pushed = 0usize;
    let mut partial = 0usize;

    for mut entry in store.entries.into_iter() {
        let to_push: Vec<String> = entry
            .clock_ids
            .iter()
            .filter(|cid| !entry.pushed_clock_ids.contains(cid))
            .cloned()
            .collect();

        if to_push.is_empty() {
            // Nothing left to do; treat as fully pushed and drop.
            fully_pushed += 1;
            continue;
        }

        let key = entry
            .task_key
            .clone()
            .unwrap_or_else(|| entry.task_id.clone());

        for clock_id in &to_push {
            let clock_cfg = match cfg.clocks.iter().find(|c| &c.id == clock_id) {
                Some(c) => c,
                None => {
                    println!(
                        "{} #{} clock '{}' missing from config — skipped",
                        "!".yellow(),
                        entry.idx,
                        clock_id
                    );
                    continue;
                }
            };

            let clock = match build_clock(clock_cfg) {
                Ok(c) => c,
                Err(e) => {
                    println!(
                        "{} #{} clock '{}' init failed: {}",
                        "✗".red(),
                        entry.idx,
                        clock_id,
                        e
                    );
                    continue;
                }
            };

            let time_entry = TimeEntry {
                task_id: entry.task_id.clone(),
                issue_key: entry.task_key.clone(),
                project_name: Some(entry.platform_project_name.clone()),
                hours: entry.hours,
                description: entry.description.clone(),
                date: entry.date,
                start_time: entry.start_time.clone(),
                end_time: entry.end_time.clone(),
            };

            match clock.log_time(&time_entry).await {
                Ok(()) => {
                    println!(
                        "{} #{} pushed [{}] {:.2}h to {}",
                        "✓".green(),
                        entry.idx,
                        key.bold(),
                        entry.hours,
                        clock_id.bold()
                    );
                    entry.pushed_clock_ids.push(clock_id.clone());
                }
                Err(e) => {
                    println!(
                        "{} #{} push to '{}' failed: {}",
                        "✗".red(),
                        entry.idx,
                        clock_id,
                        e
                    );
                }
            }
        }

        if entry.pushed_clock_ids.len() == entry.clock_ids.len() {
            fully_pushed += 1;
        } else {
            partial += 1;
            remaining.push(entry);
        }
    }

    let updated = config::models::PendingStore {
        next_idx,
        entries: remaining,
    };
    config::save_pending(&updated)?;

    if partial > 0 {
        println!(
            "{} {} fully pushed, {} partial (kept in pending for retry)",
            "■".blue(),
            fully_pushed,
            partial
        );
    } else {
        println!("{} {} pushed", "■".blue(), fully_pushed);
    }
    Ok(())
}
