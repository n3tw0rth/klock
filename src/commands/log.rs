use colored::Colorize;

use crate::config;
use crate::error::Result;
use crate::services::summary::fetch_summary;

pub async fn handle(summary: bool) -> Result<()> {
    if summary {
        show_summary().await
    } else {
        show_active_session()
    }
}

fn show_active_session() -> Result<()> {
    match config::load_session()? {
        None => {
            println!("{} No active session.", "–".dimmed());
        }
        Some(session) => {
            let elapsed = chrono::Local::now()
                .signed_duration_since(session.started_at)
                .num_minutes();
            println!(
                "{} Active session: [{}] {}",
                "▶".green(),
                session.task_id.bold(),
                session.task_title
            );
            println!(
                "  Project: {}  |  Date: {}  |  Elapsed: {}m",
                session.project_code.bold(),
                session.active_date.format("%Y-%m-%d"),
                elapsed
            );
            if let Some(start) = &session.start_time_override {
                println!("  Start: {}", start.yellow());
            }
        }
    }
    Ok(())
}

async fn show_summary() -> Result<()> {
    let cfg = config::load_config()?;
    let active_date = config::load_active_date()?;

    println!("Date: {}", active_date.format("%Y-%m-%d").to_string().bold());
    println!("{}", "─".repeat(40));

    if cfg.projects.is_empty() {
        println!("{} No projects configured.", "–".dimmed());
        return Ok(());
    }

    let rows = fetch_summary(&cfg, active_date).await;
    let mut grand_total = 0.0f32;
    let mut current_code = String::new();
    for row in &rows {
        if row.project_code != current_code {
            current_code = row.project_code.clone();
            println!("[{}] {}", row.project_code.bold(), row.project_name);
        }
        match &row.hours {
            Ok(h) => {
                grand_total += *h;
                println!("  {:<16} {:.1}h", row.clock_id, h);
            }
            Err(e) => {
                println!("  {} {}: {}", "!".yellow(), row.clock_id, e);
            }
        }
    }

    println!("{}", "─".repeat(40));
    println!("Total: {:.1}h", grand_total);

    Ok(())
}
