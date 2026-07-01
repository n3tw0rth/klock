use chrono::{Duration, Local, Timelike};

use crate::error::{KlockError, Result};

pub fn hhmm_from_now() -> String {
    let now = Local::now();
    format!("{:02}{:02}", now.hour(), now.minute())
}

pub fn parse_hhmm(input: &str) -> Result<String> {
    let trimmed = input.trim();
    if trimmed.eq_ignore_ascii_case("now") || trimmed.is_empty() {
        return Ok(hhmm_from_now());
    }
    if trimmed.len() == 4 && trimmed.chars().all(|c| c.is_ascii_digit()) {
        let h: u32 = trimmed[0..2].parse().unwrap_or(99);
        let m: u32 = trimmed[2..4].parse().unwrap_or(99);
        if h < 24 && m < 60 {
            return Ok(trimmed.to_string());
        }
    }
    Err(KlockError::ConfigError(
        "Invalid time. Use HHMM (e.g. 0930) or 'now'.".to_string(),
    ))
}

pub fn hhmm_diff_hours(start: &str, stop: &str) -> Result<f32> {
    let start_h: u32 = start.get(0..2).and_then(|h| h.parse().ok()).ok_or_else(|| {
        KlockError::SessionError(format!("Invalid start time '{start}'."))
    })?;
    let start_m: u32 = start.get(2..4).and_then(|m| m.parse().ok()).ok_or_else(|| {
        KlockError::SessionError(format!("Invalid start time '{start}'."))
    })?;
    let stop_h: u32 = stop.get(0..2).and_then(|h| h.parse().ok()).ok_or_else(|| {
        KlockError::SessionError(format!("Invalid stop time '{stop}'."))
    })?;
    let stop_m: u32 = stop.get(2..4).and_then(|m| m.parse().ok()).ok_or_else(|| {
        KlockError::SessionError(format!("Invalid stop time '{stop}'."))
    })?;

    let start_mins = (start_h * 60 + start_m) as i32;
    let stop_mins = (stop_h * 60 + stop_m) as i32;

    if stop_mins <= start_mins {
        return Err(KlockError::SessionError(
            "Stop time must be after start time.".to_string(),
        ));
    }

    Ok((stop_mins - start_mins) as f32 / 60.0)
}

pub fn format_elapsed(d: Duration) -> String {
    let total_minutes = d.num_minutes().max(0);
    let h = total_minutes / 60;
    let m = total_minutes % 60;
    if h > 0 {
        format!("{h}h {m:02}m")
    } else {
        format!("{m}m")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diff_basic() {
        assert!((hhmm_diff_hours("0900", "1100").unwrap() - 2.0).abs() < 1e-3);
    }

    #[test]
    fn diff_minutes() {
        assert!((hhmm_diff_hours("0915", "1045").unwrap() - 1.5).abs() < 1e-3);
    }

    #[test]
    fn diff_rejects_backwards() {
        assert!(hhmm_diff_hours("1100", "0900").is_err());
    }

    #[test]
    fn diff_rejects_garbage() {
        assert!(hhmm_diff_hours("ab", "1100").is_err());
    }

    #[test]
    fn parse_now_expands() {
        let s = parse_hhmm("now").unwrap();
        assert_eq!(s.len(), 4);
    }

    #[test]
    fn parse_valid_hhmm() {
        assert_eq!(parse_hhmm("0930").unwrap(), "0930");
        assert_eq!(parse_hhmm("2359").unwrap(), "2359");
    }

    #[test]
    fn parse_rejects_invalid() {
        assert!(parse_hhmm("2460").is_err());
        assert!(parse_hhmm("9999").is_err());
        assert!(parse_hhmm("abc").is_err());
        assert!(parse_hhmm("123").is_err());
    }
}
