use crate::config::{
    self,
    models::{AppConfig, BoardConfig, BoardPlatform, ClockConfig, ClockPlatform},
};
use crate::error::{KlockError, Result};

#[derive(Debug, Clone)]
pub struct BoardDraft {
    pub platform: BoardPlatform,
    pub base_url: String,
    pub email: String,
    pub team_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ClockDraft {
    pub platform: ClockPlatform,
    pub base_url: String,
    pub email: String,
}

fn board_platform_slug(p: &BoardPlatform) -> &'static str {
    match p {
        BoardPlatform::Jira => "jira",
        BoardPlatform::ClickUp => "clickup",
    }
}

fn clock_platform_slug(p: &ClockPlatform) -> &'static str {
    match p {
        ClockPlatform::Jira => "jira",
        ClockPlatform::Clockify => "clockify",
    }
}

pub fn duplicate_board_exists(cfg: &AppConfig, draft: &BoardDraft) -> bool {
    cfg.boards.iter().any(|b| {
        b.platform == draft.platform && b.base_url == draft.base_url && b.email == draft.email
    })
}

pub fn duplicate_clock_exists(cfg: &AppConfig, draft: &ClockDraft) -> bool {
    cfg.clocks.iter().any(|c| {
        c.platform == draft.platform && c.base_url == draft.base_url && c.email == draft.email
    })
}

pub fn login_board(cfg: &mut AppConfig, draft: BoardDraft, secret: &str) -> Result<String> {
    let idx = cfg.boards.len() + 1;
    let id = format!("{}-{}", board_platform_slug(&draft.platform), idx);
    config::store_credential(&format!("klock-{id}"), &draft.email, secret)?;
    cfg.boards.push(BoardConfig {
        id: id.clone(),
        platform: draft.platform,
        base_url: draft.base_url,
        email: draft.email,
        team_id: draft.team_id,
    });
    config::save_config(cfg)?;
    Ok(id)
}

pub fn login_clock(cfg: &mut AppConfig, draft: ClockDraft, secret: &str) -> Result<String> {
    let idx = cfg.clocks.len() + 1;
    let id = format!("{}-{}", clock_platform_slug(&draft.platform), idx);
    config::store_credential(&format!("klock-{id}"), &draft.email, secret)?;
    cfg.clocks.push(ClockConfig {
        id: id.clone(),
        platform: draft.platform,
        base_url: draft.base_url,
        email: draft.email,
    });
    config::save_config(cfg)?;
    Ok(id)
}

pub fn logout_board(cfg: &mut AppConfig, id: &str) -> Result<()> {
    let board = cfg
        .boards
        .iter()
        .find(|b| b.id == id)
        .ok_or_else(|| KlockError::NotFound(format!("Board '{id}' not found")))?
        .clone();
    let _ = config::delete_credential(&format!("klock-{id}"), &board.email);
    cfg.boards.retain(|b| b.id != id);
    config::save_config(cfg)
}

pub fn logout_clock(cfg: &mut AppConfig, id: &str) -> Result<()> {
    let clock = cfg
        .clocks
        .iter()
        .find(|c| c.id == id)
        .ok_or_else(|| KlockError::NotFound(format!("Clock '{id}' not found")))?
        .clone();
    let _ = config::delete_credential(&format!("klock-{id}"), &clock.email);
    cfg.clocks.retain(|c| c.id != id);
    config::save_config(cfg)
}
