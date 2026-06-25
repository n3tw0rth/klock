use chrono::NaiveDate;
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use inquire::{Confirm, DateSelect, Password, Select, Text};

use crate::error::{KlockError, Result};

pub fn fuzzy_select<T>(items: Vec<T>, label_fn: impl Fn(&T) -> String, prompt: &str) -> Result<T> {
    if items.is_empty() {
        return Err(KlockError::NotFound(format!("No results for: {prompt}")));
    }

    if items.len() == 1 {
        let label = label_fn(&items[0]);
        let confirmed = Confirm::new(&format!("Select \"{label}\"?"))
            .with_default(true)
            .prompt()
            .map_err(|e| KlockError::NotFound(e.to_string()))?;
        if confirmed {
            return Ok(items.into_iter().next().unwrap());
        } else {
            return Err(KlockError::NotFound("Selection cancelled".to_string()));
        }
    }

    let matcher = SkimMatcherV2::default();
    let query = Text::new(prompt)
        .prompt()
        .map_err(|e| KlockError::NotFound(e.to_string()))?;

    let mut scored: Vec<(i64, usize)> = items
        .iter()
        .enumerate()
        .filter_map(|(i, item)| {
            matcher
                .fuzzy_match(&label_fn(item), &query)
                .map(|score| (score, i))
        })
        .collect();

    if scored.is_empty() {
        return Err(KlockError::NotFound(format!("No matches for: {query}")));
    }

    scored.sort_by(|a, b| b.0.cmp(&a.0));

    let options: Vec<String> = scored.iter().map(|(_, i)| label_fn(&items[*i])).collect();
    let selected = Select::new(prompt, options.clone())
        .prompt()
        .map_err(|e| KlockError::NotFound(e.to_string()))?;

    let idx = scored[options.iter().position(|o| o == &selected).unwrap()].1;
    let result = items.into_iter().nth(idx).unwrap();
    Ok(result)
}

pub fn prompt_text(label: &str, default: Option<&str>) -> Result<String> {
    let mut t = Text::new(label);
    if let Some(d) = default {
        t = t.with_default(d);
    }
    t.prompt().map_err(|e| KlockError::ConfigError(e.to_string()))
}

pub fn prompt_password(label: &str) -> Result<String> {
    Password::new(label)
        .without_confirmation()
        .prompt()
        .map_err(|e| KlockError::AuthError(e.to_string()))
}

pub fn prompt_select(label: &str, options: Vec<String>) -> Result<String> {
    Select::new(label, options)
        .prompt()
        .map_err(|e| KlockError::ConfigError(e.to_string()))
}

pub fn prompt_confirm(label: &str) -> Result<bool> {
    Confirm::new(label)
        .with_default(false)
        .prompt()
        .map_err(|e| KlockError::ConfigError(e.to_string()))
}

pub fn prompt_date(label: &str) -> Result<NaiveDate> {
    DateSelect::new(label)
        .prompt()
        .map_err(|e: inquire::InquireError| KlockError::ConfigError(e.to_string()))
}

pub fn prompt_time(label: &str, default: Option<&str>) -> Result<String> {
    loop {
        let mut t = Text::new(label);
        if let Some(d) = default {
            t = t.with_default(d);
        }
        let val = t.prompt().map_err(|e| KlockError::ConfigError(e.to_string()))?;
        if val == "now" {
            let now = chrono::Local::now();
            return Ok(format!("{:02}{:02}", now.hour(), now.minute()));
        }
        if val.len() == 4 && val.chars().all(|c| c.is_ascii_digit()) {
            let h: u32 = val[0..2].parse().unwrap_or(99);
            let m: u32 = val[2..4].parse().unwrap_or(99);
            if h < 24 && m < 60 {
                return Ok(val);
            }
        }
        eprintln!("Invalid time format. Use HHMM (e.g. 0930) or 'now'.");
    }
}

// needed for prompt_time
use chrono::Timelike;
