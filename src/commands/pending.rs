use colored::Colorize;

use crate::cli::PendingAction;
use crate::config;
use crate::error::Result;
use crate::services::pending::{apply_patch, push_all, remove as remove_entry, PendingPatch};
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
    apply_patch(
        &mut store,
        idx,
        PendingPatch {
            hours,
            start,
            end,
            description,
        },
    )?;
    config::save_pending(&store)?;
    println!("{} Updated pending #{}", "✓".green(), idx);
    Ok(())
}

fn remove(idx: u32) -> Result<()> {
    let mut store = config::load_pending()?;
    remove_entry(&mut store, idx)?;
    config::save_pending(&store)?;
    println!("{} Removed pending #{}", "✓".green(), idx);
    Ok(())
}

async fn push(skip_confirm: bool) -> Result<()> {
    let mut store = config::load_pending()?;
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

    let report = push_all(&cfg, &mut store).await;
    config::save_pending(&store)?;

    for (idx, msg) in &report.errors {
        println!("{} #{} {}", "✗".red(), idx, msg);
    }

    if report.partial > 0 {
        println!(
            "{} {} fully pushed, {} partial (kept in pending for retry)",
            "■".blue(),
            report.fully,
            report.partial
        );
    } else {
        println!("{} {} pushed", "■".blue(), report.fully);
    }
    Ok(())
}
