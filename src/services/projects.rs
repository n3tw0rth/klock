use crate::config::{
    self,
    models::{AppConfig, BoardPlatform, ClockConfig, ClockPlatform, PendingStore, ProjectConfig},
};
use crate::error::{KlockError, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntegrationKind {
    Board,
    Clock,
}

impl IntegrationKind {
    pub fn label(&self) -> &'static str {
        match self {
            IntegrationKind::Board => "board",
            IntegrationKind::Clock => "clock",
        }
    }
}

#[derive(Debug, Clone)]
pub enum ClockOption {
    Existing(String),
    Derive { board_id: String },
}

pub fn clock_options_for(cfg: &AppConfig, code: &str) -> Vec<ClockOption> {
    let derive: Vec<ClockOption> = cfg
        .projects
        .iter()
        .find(|p| p.code == code)
        .map(|proj| {
            proj.board_ids
                .iter()
                .filter_map(|bid| {
                    cfg.boards
                        .iter()
                        .find(|b| &b.id == bid && b.platform == BoardPlatform::Jira)
                })
                .filter(|board| {
                    !cfg.clocks.iter().any(|c| {
                        c.platform == ClockPlatform::Jira
                            && c.base_url == board.base_url
                            && c.email == board.email
                    })
                })
                .map(|b| ClockOption::Derive {
                    board_id: b.id.clone(),
                })
                .collect()
        })
        .unwrap_or_default();

    let mut out = derive;
    out.extend(
        cfg.clocks
            .iter()
            .map(|c| ClockOption::Existing(c.id.clone())),
    );
    out
}

pub fn derive_jira_clock_from_board(cfg: &mut AppConfig, board_id: &str) -> Result<String> {
    let board = cfg
        .boards
        .iter()
        .find(|b| b.id == board_id)
        .ok_or_else(|| KlockError::NotFound(format!("Board '{board_id}' not found")))?
        .clone();

    if let Some(existing) = cfg.clocks.iter().find(|c| {
        c.platform == ClockPlatform::Jira
            && c.base_url == board.base_url
            && c.email == board.email
    }) {
        return Ok(existing.id.clone());
    }

    let jira_clock_count = cfg
        .clocks
        .iter()
        .filter(|c| c.platform == ClockPlatform::Jira)
        .count();
    let new_id = format!("jira-worklog-{}", jira_clock_count + 1);

    let secret = config::get_credential(&format!("klock-{}", board.id), &board.email)?;
    config::store_credential(&format!("klock-{new_id}"), &board.email, &secret)?;

    cfg.clocks.push(ClockConfig {
        id: new_id.clone(),
        platform: ClockPlatform::Jira,
        base_url: board.base_url.clone(),
        email: board.email.clone(),
    });

    Ok(new_id)
}

pub fn link_board_to_project(
    cfg: &mut AppConfig,
    code: &str,
    board_id: &str,
    platform_project_id: String,
    platform_project_name: String,
) -> Result<()> {
    if let Some(proj) = cfg.projects.iter_mut().find(|p| p.code == code) {
        if !proj.board_ids.iter().any(|b| b == board_id) {
            proj.board_ids.push(board_id.to_string());
        }
        proj.platform_project_id = platform_project_id;
        proj.platform_project_name = platform_project_name;
    } else {
        cfg.projects.push(ProjectConfig {
            code: code.to_string(),
            board_ids: vec![board_id.to_string()],
            clock_ids: vec![],
            platform_project_id,
            platform_project_name,
        });
    }
    config::save_config(cfg)
}

pub fn link_clock_to_project(cfg: &mut AppConfig, code: &str, clock_id: &str) -> Result<()> {
    if let Some(proj) = cfg.projects.iter_mut().find(|p| p.code == code) {
        if !proj.clock_ids.iter().any(|c| c == clock_id) {
            proj.clock_ids.push(clock_id.to_string());
        }
    } else {
        cfg.projects.push(ProjectConfig {
            code: code.to_string(),
            board_ids: vec![],
            clock_ids: vec![clock_id.to_string()],
            platform_project_id: String::new(),
            platform_project_name: String::new(),
        });
    }
    config::save_config(cfg)
}

pub fn unlink_integration(
    cfg: &mut AppConfig,
    code: &str,
    kind: IntegrationKind,
    target_id: &str,
) -> Result<()> {
    let proj_idx = cfg
        .projects
        .iter()
        .position(|p| p.code.eq_ignore_ascii_case(code))
        .ok_or_else(|| KlockError::NotFound(format!("Project '{code}' not found")))?;
    let project = &mut cfg.projects[proj_idx];
    match kind {
        IntegrationKind::Board => project.board_ids.retain(|id| id != target_id),
        IntegrationKind::Clock => project.clock_ids.retain(|id| id != target_id),
    }
    config::save_config(cfg)
}

pub fn pending_affected_by_clock_unlink(pending: &PendingStore, clock_id: &str) -> Vec<u32> {
    pending
        .entries
        .iter()
        .filter(|e| e.clock_ids.iter().any(|cid| cid == clock_id))
        .map(|e| e.idx)
        .collect()
}
