use crate::clocks::{build_clock, TimeEntry};
use crate::config::models::{AppConfig, PendingStore};
use crate::error::{KlockError, Result};

#[derive(Debug, Clone, Default)]
pub struct PendingPatch {
    pub hours: Option<f32>,
    pub start: Option<String>,
    pub end: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Default)]
pub struct PushReport {
    pub fully: usize,
    pub partial: usize,
    pub errors: Vec<(u32, String)>,
}

pub fn apply_patch(store: &mut PendingStore, idx: u32, patch: PendingPatch) -> Result<()> {
    let entry = store
        .entries
        .iter_mut()
        .find(|e| e.idx == idx)
        .ok_or_else(|| KlockError::NotFound(format!("Pending entry #{idx} not found")))?;

    if !entry.pushed_clock_ids.is_empty() {
        return Err(KlockError::ConfigError(format!(
            "Pending #{idx} has already been partially pushed to {}. Remove it or finish the push instead of editing.",
            entry.pushed_clock_ids.join(", ")
        )));
    }

    if let Some(h) = patch.hours {
        entry.hours = h;
    }
    if let Some(s) = patch.start {
        entry.start_time = Some(s);
    }
    if let Some(e) = patch.end {
        entry.end_time = Some(e);
    }
    if let Some(d) = patch.description {
        entry.description = d;
    }
    Ok(())
}

pub fn remove(store: &mut PendingStore, idx: u32) -> Result<()> {
    let before = store.entries.len();
    store.entries.retain(|e| e.idx != idx);
    if store.entries.len() == before {
        return Err(KlockError::NotFound(format!(
            "Pending entry #{idx} not found"
        )));
    }
    Ok(())
}

pub async fn push_all(cfg: &AppConfig, store: &mut PendingStore) -> PushReport {
    let next_idx = store.next_idx;
    let entries = std::mem::take(&mut store.entries);
    let mut remaining = Vec::new();
    let mut report = PushReport::default();

    for mut entry in entries.into_iter() {
        let to_push: Vec<String> = entry
            .clock_ids
            .iter()
            .filter(|cid| !entry.pushed_clock_ids.contains(cid))
            .cloned()
            .collect();

        if to_push.is_empty() {
            report.fully += 1;
            continue;
        }

        for clock_id in &to_push {
            let Some(clock_cfg) = cfg.clocks.iter().find(|c| &c.id == clock_id) else {
                report
                    .errors
                    .push((entry.idx, format!("clock '{clock_id}' missing from config")));
                continue;
            };
            let clock = match build_clock(clock_cfg) {
                Ok(c) => c,
                Err(e) => {
                    report
                        .errors
                        .push((entry.idx, format!("clock '{clock_id}' init failed: {e}")));
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
                Ok(()) => entry.pushed_clock_ids.push(clock_id.clone()),
                Err(e) => report
                    .errors
                    .push((entry.idx, format!("'{clock_id}' failed: {e}"))),
            }
        }

        if entry.pushed_clock_ids.len() == entry.clock_ids.len() {
            report.fully += 1;
        } else {
            report.partial += 1;
            remaining.push(entry);
        }
    }

    store.next_idx = next_idx;
    store.entries = remaining;
    report
}
