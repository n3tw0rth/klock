pub mod models;

use chrono::NaiveDate;
use keyring::Entry;
use std::path::PathBuf;

use crate::error::{KlockError, Result};
use models::{AppConfig, PendingEntry, PendingStore, Session};

pub fn config_path() -> PathBuf {
    dirs::home_dir()
        .expect("home dir not found")
        .join(".klock")
        .join("config.toml")
}

pub fn session_path() -> PathBuf {
    dirs::home_dir()
        .expect("home dir not found")
        .join(".klock")
        .join("session.toml")
}

pub fn state_path() -> PathBuf {
    dirs::home_dir()
        .expect("home dir not found")
        .join(".klock")
        .join("state.toml")
}

pub fn pending_path() -> PathBuf {
    dirs::home_dir()
        .expect("home dir not found")
        .join(".klock")
        .join("pending.toml")
}

fn ensure_dir(path: &PathBuf) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| KlockError::ConfigError(e.to_string()))?;
    }
    Ok(())
}

pub fn load_config() -> Result<AppConfig> {
    let path = config_path();
    if !path.exists() {
        return Ok(AppConfig::default());
    }
    let content =
        std::fs::read_to_string(&path).map_err(|e| KlockError::ConfigError(e.to_string()))?;
    toml::from_str(&content).map_err(|e| KlockError::ConfigError(e.to_string()))
}

pub fn save_config(config: &AppConfig) -> Result<()> {
    let path = config_path();
    ensure_dir(&path)?;
    let content =
        toml::to_string_pretty(config).map_err(|e| KlockError::ConfigError(e.to_string()))?;
    std::fs::write(&path, content).map_err(|e| KlockError::ConfigError(e.to_string()))
}

pub fn load_session() -> Result<Option<Session>> {
    let path = session_path();
    if !path.exists() {
        return Ok(None);
    }
    let content =
        std::fs::read_to_string(&path).map_err(|e| KlockError::SessionError(e.to_string()))?;
    let session = toml::from_str(&content).map_err(|e| KlockError::SessionError(e.to_string()))?;
    Ok(Some(session))
}

pub fn save_session(session: &Session) -> Result<()> {
    let path = session_path();
    ensure_dir(&path)?;
    let content =
        toml::to_string_pretty(session).map_err(|e| KlockError::SessionError(e.to_string()))?;
    std::fs::write(&path, content).map_err(|e| KlockError::SessionError(e.to_string()))
}

pub fn clear_session() -> Result<()> {
    let path = session_path();
    if path.exists() {
        std::fs::remove_file(&path).map_err(|e| KlockError::SessionError(e.to_string()))?;
    }
    Ok(())
}

pub fn load_active_date() -> Result<NaiveDate> {
    let path = state_path();
    if !path.exists() {
        return Ok(chrono::Local::now().date_naive());
    }
    let content =
        std::fs::read_to_string(&path).map_err(|e| KlockError::ConfigError(e.to_string()))?;
    let map: toml::Table =
        toml::from_str(&content).map_err(|e| KlockError::ConfigError(e.to_string()))?;
    if let Some(toml::Value::String(s)) = map.get("active_date") {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").map_err(|e| KlockError::ConfigError(e.to_string()))
    } else {
        Ok(chrono::Local::now().date_naive())
    }
}

pub fn save_active_date(date: NaiveDate) -> Result<()> {
    let path = state_path();
    ensure_dir(&path)?;
    let content = format!("active_date = \"{}\"\n", date.format("%Y-%m-%d"));
    std::fs::write(&path, content).map_err(|e| KlockError::ConfigError(e.to_string()))
}

pub fn load_pending() -> Result<PendingStore> {
    let path = pending_path();
    if !path.exists() {
        return Ok(PendingStore::default());
    }
    let content =
        std::fs::read_to_string(&path).map_err(|e| KlockError::ConfigError(e.to_string()))?;
    let mut store: PendingStore =
        toml::from_str(&content).map_err(|e| KlockError::ConfigError(e.to_string()))?;
    for entry in &mut store.entries {
        if entry.clock_ids.is_empty() {
            if let Some(legacy) = entry.clock_id.take() {
                entry.clock_ids.push(legacy);
            }
        } else {
            entry.clock_id = None;
        }
    }
    Ok(store)
}

pub fn save_pending(store: &PendingStore) -> Result<()> {
    let path = pending_path();
    ensure_dir(&path)?;
    let content =
        toml::to_string_pretty(store).map_err(|e| KlockError::ConfigError(e.to_string()))?;
    std::fs::write(&path, content).map_err(|e| KlockError::ConfigError(e.to_string()))
}

pub fn append_pending(mut entry: PendingEntry) -> Result<u32> {
    let mut store = load_pending()?;
    let idx = store.next_idx.max(1);
    entry.idx = idx;
    store.next_idx = idx + 1;
    store.entries.push(entry);
    save_pending(&store)?;
    Ok(idx)
}

pub fn store_credential(service: &str, account: &str, secret: &str) -> Result<()> {
    let entry = Entry::new(service, account).map_err(|e| KlockError::AuthError(e.to_string()))?;
    entry
        .set_password(secret)
        .map_err(|e| KlockError::AuthError(e.to_string()))
}

pub fn get_credential(service: &str, account: &str) -> Result<String> {
    let entry = Entry::new(service, account).map_err(|e| KlockError::AuthError(e.to_string()))?;
    entry.get_password().map_err(|_| {
        KlockError::AuthError(format!(
            "No credential found for {service}/{account}. Run `klock auth login` first."
        ))
    })
}

pub fn delete_credential(service: &str, account: &str) -> Result<()> {
    let entry = Entry::new(service, account).map_err(|e| KlockError::AuthError(e.to_string()))?;
    entry
        .delete_password()
        .map_err(|e: keyring::Error| KlockError::AuthError(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use models::{BoardConfig, BoardPlatform};

    #[test]
    fn test_config_roundtrip() {
        let mut config = AppConfig::default();
        config.boards.push(BoardConfig {
            id: "jira-test".to_string(),
            platform: BoardPlatform::Jira,
            base_url: "https://test.atlassian.net".to_string(),
            email: "test@example.com".to_string(),
            team_id: None,
        });
        let serialized = toml::to_string_pretty(&config).unwrap();
        let deserialized: AppConfig = toml::from_str(&serialized).unwrap();
        assert_eq!(deserialized.boards.len(), 1);
        assert_eq!(deserialized.boards[0].id, "jira-test");
    }
}
