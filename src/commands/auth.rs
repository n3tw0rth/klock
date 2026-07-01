use colored::Colorize;

use crate::cli::AuthAction;
use crate::config::{
    self,
    models::{BoardPlatform, ClockPlatform},
};
use crate::error::Result;
use crate::services::auth::{
    duplicate_board_exists, duplicate_clock_exists, login_board, login_clock, logout_board,
    logout_clock, BoardDraft, ClockDraft,
};
use crate::session::{prompt_confirm, prompt_password, prompt_select, prompt_text};

pub async fn handle(action: AuthAction) -> Result<()> {
    match action {
        AuthAction::Login => login().await,
        AuthAction::Logout => logout().await,
    }
}

async fn login() -> Result<()> {
    let kind = prompt_select("Login to:", vec!["Board".to_string(), "Clock".to_string()])?;
    let mut cfg = config::load_config()?;

    if kind == "Board" {
        let platform_str =
            prompt_select("Platform:", vec!["Jira".to_string(), "ClickUp".to_string()])?;
        let platform = if platform_str == "Jira" {
            BoardPlatform::Jira
        } else {
            BoardPlatform::ClickUp
        };

        let base_url = if platform == BoardPlatform::ClickUp {
            "https://api.clickup.com".to_string()
        } else {
            prompt_text("Base URL (e.g. https://org.atlassian.net):", None)?
        };
        let team_id = if platform == BoardPlatform::ClickUp {
            Some(prompt_text("ClickUp Team ID:", None)?)
        } else {
            None
        };
        let email = prompt_text("Email:", None)?;
        let api_token = prompt_password("API Token:")?;

        let draft = BoardDraft {
            platform,
            base_url,
            email,
            team_id,
        };
        if duplicate_board_exists(&cfg, &draft)
            && !prompt_confirm(
                "An integration with this platform/URL/email already exists. Add another?",
            )?
        {
            return Ok(());
        }

        let id = login_board(&mut cfg, draft, &api_token)?;
        println!("{} Board integration '{}' saved.", "✓".green(), id);
    } else {
        let platform_str =
            prompt_select("Platform:", vec!["Jira".to_string(), "Clockify".to_string()])?;
        let platform = if platform_str == "Jira" {
            ClockPlatform::Jira
        } else {
            ClockPlatform::Clockify
        };

        let base_url = if platform == ClockPlatform::Clockify {
            "https://api.clockify.me".to_string()
        } else {
            prompt_text("Base URL (e.g. https://org.atlassian.net):", None)?
        };
        let email = prompt_text("Email:", None)?;
        let api_token = prompt_password("API Key / Token:")?;

        let draft = ClockDraft {
            platform,
            base_url,
            email,
        };
        if duplicate_clock_exists(&cfg, &draft)
            && !prompt_confirm(
                "An integration with this platform/URL/email already exists. Add another?",
            )?
        {
            return Ok(());
        }

        let id = login_clock(&mut cfg, draft, &api_token)?;
        println!("{} Clock integration '{}' saved.", "✓".green(), id);
    }

    Ok(())
}

async fn logout() -> Result<()> {
    let kind = prompt_select("Logout from:", vec!["Board".to_string(), "Clock".to_string()])?;
    let mut cfg = config::load_config()?;

    if kind == "Board" {
        if cfg.boards.is_empty() {
            println!("{} No board integrations configured.", "!".yellow());
            return Ok(());
        }
        let ids: Vec<String> = cfg.boards.iter().map(|b| b.id.clone()).collect();
        let selected = if ids.len() == 1 {
            if !prompt_confirm(&format!("Logout from '{}'?", ids[0]))? {
                return Ok(());
            }
            ids[0].clone()
        } else {
            prompt_select("Select integration to remove:", ids)?
        };
        logout_board(&mut cfg, &selected)?;
        println!("{} Logged out from '{}'.", "✓".green(), selected);
    } else {
        if cfg.clocks.is_empty() {
            println!("{} No clock integrations configured.", "!".yellow());
            return Ok(());
        }
        let ids: Vec<String> = cfg.clocks.iter().map(|c| c.id.clone()).collect();
        let selected = if ids.len() == 1 {
            if !prompt_confirm(&format!("Logout from '{}'?", ids[0]))? {
                return Ok(());
            }
            ids[0].clone()
        } else {
            prompt_select("Select integration to remove:", ids)?
        };
        logout_clock(&mut cfg, &selected)?;
        println!("{} Logged out from '{}'.", "✓".green(), selected);
    }

    Ok(())
}
