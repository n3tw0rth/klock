use tokio::sync::mpsc;

use crate::config;
use crate::services as svc;
use crate::tui::messages::{ServiceCommand, ServiceResult};

pub async fn run(
    mut rx: mpsc::Receiver<ServiceCommand>,
    tx: mpsc::Sender<ServiceResult>,
) {
    while let Some(cmd) = rx.recv().await {
        let tx = tx.clone();
        tokio::spawn(async move {
            let result = handle(cmd).await;
            let _ = tx.send(result).await;
        });
    }
}

async fn handle(cmd: ServiceCommand) -> ServiceResult {
    match cmd {
        ServiceCommand::FetchSummary { req_id, date } => match config::load_config() {
            Ok(cfg) => {
                let rows = svc::summary::fetch_summary(&cfg, date).await;
                ServiceResult::SummaryReady { req_id, date, rows }
            }
            Err(e) => ServiceResult::Error {
                req_id,
                message: format!("load config: {e}"),
            },
        },
        ServiceCommand::StartSession { req_id, session } => match config::save_session(&session) {
            Ok(()) => ServiceResult::SessionStarted { req_id, session },
            Err(e) => ServiceResult::Error {
                req_id,
                message: format!("save session: {e}"),
            },
        },
        ServiceCommand::StopSession { req_id, stop_time } => stop_session(req_id, stop_time).await,
        ServiceCommand::PushPending { req_id } => push_pending(req_id).await,
        ServiceCommand::EditPending {
            req_id,
            idx,
            patch,
        } => edit_pending(req_id, idx, patch),
        ServiceCommand::RemovePending { req_id, idx } => remove_pending(req_id, idx),
        ServiceCommand::SetActiveDate { req_id, date } => match svc::sessions::set_active_date(date)
        {
            Ok(()) => ServiceResult::ActiveDateChanged { req_id, date },
            Err(e) => ServiceResult::Error {
                req_id,
                message: format!("set date: {e}"),
            },
        },
        ServiceCommand::SearchProjects {
            req_id,
            board_id,
            query,
        } => search_projects(req_id, board_id, query).await,
        ServiceCommand::SearchTasks {
            req_id,
            project_code,
            query,
        } => search_tasks(req_id, project_code, query).await,
        ServiceCommand::AddProjectBoard {
            req_id,
            code,
            board_id,
            remote,
        } => add_project_board(req_id, code, board_id, remote),
        ServiceCommand::AddProjectClock {
            req_id,
            code,
            clock_choice,
        } => add_project_clock(req_id, code, clock_choice),
        ServiceCommand::RemoveProjectIntegration {
            req_id,
            code,
            kind,
            target_id,
        } => remove_project_integration(req_id, code, kind, target_id),
        ServiceCommand::AuthLoginBoard {
            req_id,
            draft,
            secret,
        } => auth_login_board(req_id, draft, secret),
        ServiceCommand::AuthLoginClock {
            req_id,
            draft,
            secret,
        } => auth_login_clock(req_id, draft, secret),
        ServiceCommand::AuthLogoutBoard { req_id, id } => auth_logout_board(req_id, id),
        ServiceCommand::AuthLogoutClock { req_id, id } => auth_logout_clock(req_id, id),
    }
}

async fn stop_session(req_id: u64, stop_time: String) -> ServiceResult {
    let session = match config::load_session() {
        Ok(Some(s)) => s,
        Ok(None) => {
            return ServiceResult::Error {
                req_id,
                message: "no active session".into(),
            }
        }
        Err(e) => {
            return ServiceResult::Error {
                req_id,
                message: format!("load session: {e}"),
            }
        }
    };
    let cfg = match config::load_config() {
        Ok(c) => c,
        Err(e) => {
            return ServiceResult::Error {
                req_id,
                message: format!("load config: {e}"),
            }
        }
    };
    let project = match cfg
        .projects
        .iter()
        .find(|p| p.code == session.project_code)
    {
        Some(p) => p,
        None => {
            return ServiceResult::Error {
                req_id,
                message: format!("project '{}' not found", session.project_code),
            }
        }
    };
    let entry = match svc::sessions::build_pending_entry(&session, project, &stop_time) {
        Ok(e) => e,
        Err(e) => {
            return ServiceResult::Error {
                req_id,
                message: e.to_string(),
            }
        }
    };
    let hours = entry.hours;
    let idx = match config::append_pending(entry) {
        Ok(i) => i,
        Err(e) => {
            return ServiceResult::Error {
                req_id,
                message: format!("queue: {e}"),
            }
        }
    };
    if let Err(e) = config::clear_session() {
        return ServiceResult::Error {
            req_id,
            message: format!("clear session: {e}"),
        };
    }
    match config::load_pending() {
        Ok(pending) => ServiceResult::SessionStopped {
            req_id,
            queued_idx: idx,
            hours,
            pending,
        },
        Err(e) => ServiceResult::Error {
            req_id,
            message: format!("reload pending: {e}"),
        },
    }
}

async fn push_pending(req_id: u64) -> ServiceResult {
    let mut store = match config::load_pending() {
        Ok(s) => s,
        Err(e) => {
            return ServiceResult::Error {
                req_id,
                message: format!("load pending: {e}"),
            }
        }
    };
    let cfg = match config::load_config() {
        Ok(c) => c,
        Err(e) => {
            return ServiceResult::Error {
                req_id,
                message: format!("load config: {e}"),
            }
        }
    };
    let report = svc::pending::push_all(&cfg, &mut store).await;
    if let Err(e) = config::save_pending(&store) {
        return ServiceResult::Error {
            req_id,
            message: format!("save pending: {e}"),
        };
    }
    ServiceResult::PushReportReady {
        req_id,
        report,
        store,
    }
}

fn edit_pending(
    req_id: u64,
    idx: u32,
    patch: crate::services::pending::PendingPatch,
) -> ServiceResult {
    let mut store = match config::load_pending() {
        Ok(s) => s,
        Err(e) => {
            return ServiceResult::Error {
                req_id,
                message: format!("load pending: {e}"),
            }
        }
    };
    if let Err(e) = svc::pending::apply_patch(&mut store, idx, patch) {
        return ServiceResult::Error {
            req_id,
            message: e.to_string(),
        };
    }
    if let Err(e) = config::save_pending(&store) {
        return ServiceResult::Error {
            req_id,
            message: format!("save pending: {e}"),
        };
    }
    ServiceResult::PendingChanged { req_id, store }
}

fn remove_pending(req_id: u64, idx: u32) -> ServiceResult {
    let mut store = match config::load_pending() {
        Ok(s) => s,
        Err(e) => {
            return ServiceResult::Error {
                req_id,
                message: format!("load pending: {e}"),
            }
        }
    };
    if let Err(e) = svc::pending::remove(&mut store, idx) {
        return ServiceResult::Error {
            req_id,
            message: e.to_string(),
        };
    }
    if let Err(e) = config::save_pending(&store) {
        return ServiceResult::Error {
            req_id,
            message: format!("save pending: {e}"),
        };
    }
    ServiceResult::PendingChanged { req_id, store }
}

async fn search_projects(req_id: u64, board_id: String, query: String) -> ServiceResult {
    let cfg = match config::load_config() {
        Ok(c) => c,
        Err(e) => {
            return ServiceResult::Error {
                req_id,
                message: format!("load config: {e}"),
            }
        }
    };
    let Some(board_cfg) = cfg.boards.iter().find(|b| b.id == board_id) else {
        return ServiceResult::Error {
            req_id,
            message: format!("board '{board_id}' not found"),
        };
    };
    let board = match crate::boards::build_board(board_cfg) {
        Ok(b) => b,
        Err(e) => {
            return ServiceResult::Error {
                req_id,
                message: format!("board init: {e}"),
            }
        }
    };
    match board.search_projects(&query).await {
        Ok(results) => ServiceResult::ProjectsFound { req_id, results },
        Err(e) => ServiceResult::Error {
            req_id,
            message: format!("search projects: {e}"),
        },
    }
}

async fn search_tasks(req_id: u64, project_code: String, query: String) -> ServiceResult {
    let cfg = match config::load_config() {
        Ok(c) => c,
        Err(e) => {
            return ServiceResult::Error {
                req_id,
                message: format!("load config: {e}"),
            }
        }
    };
    let project = match svc::sessions::resolve_project(&cfg, &project_code) {
        Ok(p) => p.clone(),
        Err(e) => {
            return ServiceResult::Error {
                req_id,
                message: e.to_string(),
            }
        }
    };
    match svc::sessions::perform_search_tasks(&cfg, &project, &query).await {
        Ok(results) => ServiceResult::TasksFound { req_id, results },
        Err(e) => ServiceResult::Error {
            req_id,
            message: e.to_string(),
        },
    }
}

fn add_project_board(
    req_id: u64,
    code: String,
    board_id: String,
    remote: crate::boards::RemoteProject,
) -> ServiceResult {
    let mut cfg = match config::load_config() {
        Ok(c) => c,
        Err(e) => {
            return ServiceResult::Error {
                req_id,
                message: format!("load config: {e}"),
            }
        }
    };
    if let Err(e) =
        svc::projects::link_board_to_project(&mut cfg, &code, &board_id, remote.id, remote.name)
    {
        return ServiceResult::Error {
            req_id,
            message: e.to_string(),
        };
    }
    ServiceResult::ConfigChanged { req_id, config: cfg }
}

fn add_project_clock(
    req_id: u64,
    code: String,
    clock_choice: crate::tui::messages::ClockChoice,
) -> ServiceResult {
    let mut cfg = match config::load_config() {
        Ok(c) => c,
        Err(e) => {
            return ServiceResult::Error {
                req_id,
                message: format!("load config: {e}"),
            }
        }
    };
    let clock_id = match clock_choice {
        crate::tui::messages::ClockChoice::Existing(id) => id,
        crate::tui::messages::ClockChoice::DeriveFromBoard(board_id) => {
            match svc::projects::derive_jira_clock_from_board(&mut cfg, &board_id) {
                Ok(id) => id,
                Err(e) => {
                    return ServiceResult::Error {
                        req_id,
                        message: e.to_string(),
                    }
                }
            }
        }
    };
    if let Err(e) = svc::projects::link_clock_to_project(&mut cfg, &code, &clock_id) {
        return ServiceResult::Error {
            req_id,
            message: e.to_string(),
        };
    }
    ServiceResult::ConfigChanged { req_id, config: cfg }
}

fn remove_project_integration(
    req_id: u64,
    code: String,
    kind: crate::services::projects::IntegrationKind,
    target_id: String,
) -> ServiceResult {
    let mut cfg = match config::load_config() {
        Ok(c) => c,
        Err(e) => {
            return ServiceResult::Error {
                req_id,
                message: format!("load config: {e}"),
            }
        }
    };
    if let Err(e) = svc::projects::unlink_integration(&mut cfg, &code, kind, &target_id) {
        return ServiceResult::Error {
            req_id,
            message: e.to_string(),
        };
    }
    ServiceResult::ConfigChanged { req_id, config: cfg }
}

fn auth_login_board(
    req_id: u64,
    draft: crate::services::auth::BoardDraft,
    secret: String,
) -> ServiceResult {
    let mut cfg = match config::load_config() {
        Ok(c) => c,
        Err(e) => {
            return ServiceResult::Error {
                req_id,
                message: format!("load config: {e}"),
            }
        }
    };
    if let Err(e) = svc::auth::login_board(&mut cfg, draft, &secret) {
        return ServiceResult::Error {
            req_id,
            message: e.to_string(),
        };
    }
    ServiceResult::ConfigChanged { req_id, config: cfg }
}

fn auth_login_clock(
    req_id: u64,
    draft: crate::services::auth::ClockDraft,
    secret: String,
) -> ServiceResult {
    let mut cfg = match config::load_config() {
        Ok(c) => c,
        Err(e) => {
            return ServiceResult::Error {
                req_id,
                message: format!("load config: {e}"),
            }
        }
    };
    if let Err(e) = svc::auth::login_clock(&mut cfg, draft, &secret) {
        return ServiceResult::Error {
            req_id,
            message: e.to_string(),
        };
    }
    ServiceResult::ConfigChanged { req_id, config: cfg }
}

fn auth_logout_board(req_id: u64, id: String) -> ServiceResult {
    let mut cfg = match config::load_config() {
        Ok(c) => c,
        Err(e) => {
            return ServiceResult::Error {
                req_id,
                message: format!("load config: {e}"),
            }
        }
    };
    if let Err(e) = svc::auth::logout_board(&mut cfg, &id) {
        return ServiceResult::Error {
            req_id,
            message: e.to_string(),
        };
    }
    ServiceResult::ConfigChanged { req_id, config: cfg }
}

fn auth_logout_clock(req_id: u64, id: String) -> ServiceResult {
    let mut cfg = match config::load_config() {
        Ok(c) => c,
        Err(e) => {
            return ServiceResult::Error {
                req_id,
                message: format!("load config: {e}"),
            }
        }
    };
    if let Err(e) = svc::auth::logout_clock(&mut cfg, &id) {
        return ServiceResult::Error {
            req_id,
            message: e.to_string(),
        };
    }
    ServiceResult::ConfigChanged { req_id, config: cfg }
}
