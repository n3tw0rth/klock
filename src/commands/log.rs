use colored::Colorize;

use crate::clocks::build_clock;
use crate::config;
use crate::error::Result;

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

    let mut grand_total = 0.0f32;

    for project in &cfg.projects {
        for clock_id in &project.clock_ids {
            if let Some(clock_cfg) = cfg.clocks.iter().find(|c| &c.id == clock_id) {
                match build_clock(clock_cfg) {
                    Ok(clock) => {
                        match clock
                            .get_logged_time(&project.platform_project_id, active_date)
                            .await
                        {
                            Ok(hours) => {
                                grand_total += hours;
                                println!(
                                    "[{}] {}",
                                    project.code.bold(),
                                    project.platform_project_name
                                );
                                println!("  {:<16} {:.1}h", clock_id, hours);
                            }
                            Err(e) => {
                                println!(
                                    "  {} Could not fetch from {}: {}",
                                    "!".yellow(),
                                    clock_id,
                                    e
                                );
                            }
                        }
                    }
                    Err(e) => {
                        println!("  {} Clock '{}' error: {}", "!".yellow(), clock_id, e);
                    }
                }
            }
        }
    }

    println!("{}", "─".repeat(40));
    println!("Total: {:.1}h", grand_total);

    Ok(())
}
