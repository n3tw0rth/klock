use chrono::NaiveDate;
use colored::Colorize;

use crate::config;
use crate::error::{JiredError, Result};
use crate::session::prompt_date;

pub async fn handle(date: Option<String>) -> Result<()> {
    let active_date = match date {
        Some(s) => NaiveDate::parse_from_str(&s, "%Y-%m-%d")
            .map_err(|_| JiredError::ConfigError(format!("Invalid date '{s}'. Use YYYY-MM-DD.")))?,
        None => prompt_date("Date:")?,
    };

    if let Some(mut session) = config::load_session()? {
        session.active_date = active_date;
        config::save_session(&session)?;
    }

    config::save_active_date(active_date)?;
    println!(
        "{} Active date set to {}",
        "✓".green(),
        active_date.format("%Y-%m-%d").to_string().bold()
    );

    Ok(())
}
