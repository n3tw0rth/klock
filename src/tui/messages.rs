use chrono::NaiveDate;

use crate::boards::{RemoteProject, RemoteTask};
use crate::config::models::{AppConfig, PendingStore, Session};
use crate::services::auth::{BoardDraft, ClockDraft};
use crate::services::pending::{PendingPatch, PushReport};
use crate::services::projects::IntegrationKind;
use crate::services::summary::SummaryRow;

#[derive(Debug)]
pub enum ServiceCommand {
    FetchSummary {
        req_id: u64,
        date: NaiveDate,
    },
    StartSession {
        req_id: u64,
        session: Session,
    },
    StopSession {
        req_id: u64,
        stop_time: String,
    },
    PushPending {
        req_id: u64,
    },
    EditPending {
        req_id: u64,
        idx: u32,
        patch: PendingPatch,
    },
    RemovePending {
        req_id: u64,
        idx: u32,
    },
    SetActiveDate {
        req_id: u64,
        date: NaiveDate,
    },
    SearchProjects {
        req_id: u64,
        board_id: String,
        query: String,
    },
    SearchTasks {
        req_id: u64,
        project_code: String,
        query: String,
    },
    AddProjectBoard {
        req_id: u64,
        code: String,
        board_id: String,
        remote: RemoteProject,
    },
    AddProjectClock {
        req_id: u64,
        code: String,
        clock_choice: ClockChoice,
    },
    RemoveProjectIntegration {
        req_id: u64,
        code: String,
        kind: IntegrationKind,
        target_id: String,
    },
    AuthLoginBoard {
        req_id: u64,
        draft: BoardDraft,
        secret: String,
    },
    AuthLoginClock {
        req_id: u64,
        draft: ClockDraft,
        secret: String,
    },
    AuthLogoutBoard {
        req_id: u64,
        id: String,
    },
    AuthLogoutClock {
        req_id: u64,
        id: String,
    },
}

#[derive(Debug, Clone)]
pub enum ClockChoice {
    Existing(String),
    DeriveFromBoard(String),
}

#[derive(Debug)]
pub enum ServiceResult {
    SummaryReady {
        req_id: u64,
        date: NaiveDate,
        rows: Vec<SummaryRow>,
    },
    SessionStarted {
        req_id: u64,
        session: Session,
    },
    SessionStopped {
        req_id: u64,
        queued_idx: u32,
        hours: f32,
        pending: PendingStore,
    },
    PendingChanged {
        req_id: u64,
        store: PendingStore,
    },
    PushReportReady {
        req_id: u64,
        report: PushReport,
        store: PendingStore,
    },
    ConfigChanged {
        req_id: u64,
        config: AppConfig,
    },
    ActiveDateChanged {
        req_id: u64,
        date: NaiveDate,
    },
    ProjectsFound {
        req_id: u64,
        results: Vec<RemoteProject>,
    },
    TasksFound {
        req_id: u64,
        results: Vec<RemoteTask>,
    },
    Error {
        req_id: u64,
        message: String,
    },
}
