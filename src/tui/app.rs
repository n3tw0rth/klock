use std::collections::HashMap;

use chrono::NaiveDate;
use ratatui::widgets::ListState;
use tokio::sync::mpsc;
use tui_input::Input;

use crate::config::models::{AppConfig, PendingStore, Session};
use crate::services::pending::PendingPatch;
use crate::services::projects::{clock_options_for, ClockOption, IntegrationKind};
use crate::services::sessions::{build_session, first_board_for_project, resolve_project};
use crate::tui::event::Event;
use crate::tui::keymap::Action;
use crate::tui::messages::{ClockChoice, ServiceCommand, ServiceResult};
use crate::tui::modal::{
    AuthDraft, AuthFieldFocus, ConfirmAction, EditPendingFocus, IntegrationRef, Modal,
    PendingEditDraft, SessionDraft,
};
use crate::tui::state::{
    clamp_list_state, clamp_table_state, DashboardState, PendingUiState, ProjectsState,
    SettingsFocus, SettingsState, Tab,
};
use crate::tui::widgets::date_picker::{self, DatePickerState};
use crate::tui::widgets::spinner::SpinnerState;
use crate::tui::widgets::time_input::TimeInputState;
use crate::tui::widgets::toast::Toast;
use crate::tui::widgets::{fuzzy_select, scroll_list, text_input, time_input};

#[derive(Debug, Clone, Copy)]
pub enum InflightKind {
    FetchSummary,
    StartSession,
    StopSession,
    PushPending,
    EditPending,
    RemovePending,
    SetActiveDate,
    SearchProjects,
    SearchTasks,
    AddProject,
    RemoveProjectIntegration,
    AuthLogin,
    AuthLogout,
}

pub struct App {
    pub config: AppConfig,
    pub pending: PendingStore,
    pub session: Option<Session>,
    pub active_date: NaiveDate,

    pub tab: Tab,
    pub dashboard: DashboardState,
    pub projects: ProjectsState,
    pub pending_ui: PendingUiState,
    pub settings_ui: SettingsState,

    pub modals: Vec<Modal>,
    pub toast: Option<Toast>,
    pub spinner: SpinnerState,

    pub inflight: HashMap<u64, InflightKind>,
    pub next_req_id: u64,

    pub cmd_tx: mpsc::Sender<ServiceCommand>,
    pub should_quit: bool,
}

impl App {
    pub fn new(
        cmd_tx: mpsc::Sender<ServiceCommand>,
        config: AppConfig,
        pending: PendingStore,
        session: Option<Session>,
        active_date: NaiveDate,
    ) -> Self {
        Self {
            config,
            pending,
            session,
            active_date,
            tab: Tab::Dashboard,
            dashboard: DashboardState::default(),
            projects: ProjectsState::default(),
            pending_ui: PendingUiState::default(),
            settings_ui: SettingsState::default(),
            modals: Vec::new(),
            toast: None,
            spinner: SpinnerState::default(),
            inflight: HashMap::new(),
            next_req_id: 0,
            cmd_tx,
            should_quit: false,
        }
    }

    pub fn is_running(&self) -> bool {
        !self.should_quit
    }

    pub fn issue_req(&mut self, kind: InflightKind) -> u64 {
        self.next_req_id = self.next_req_id.wrapping_add(1);
        let id = self.next_req_id;
        self.inflight.insert(id, kind);
        id
    }

    pub fn cancel_inflight(&mut self, req_id: u64) {
        self.inflight.remove(&req_id);
    }

    fn send(&self, cmd: ServiceCommand) {
        let tx = self.cmd_tx.clone();
        tokio::spawn(async move {
            let _ = tx.send(cmd).await;
        });
    }

    pub fn kickoff_summary_fetch(&mut self) {
        if self.dashboard.summary_loading {
            return;
        }
        let req_id = self.issue_req(InflightKind::FetchSummary);
        self.dashboard.summary_loading = true;
        self.send(ServiceCommand::FetchSummary {
            req_id,
            date: self.active_date,
        });
    }

    pub fn on_event(&mut self, ev: Event) {
        match ev {
            Event::Tick => self.tick(),
            Event::Resize(_, _) => {}
            Event::Key(key) => self.on_key(key),
            Event::Service(msg) => self.on_service(msg),
        }
    }

    fn tick(&mut self) {
        self.spinner.advance();
        if let Some(t) = &self.toast {
            if t.is_expired() {
                self.toast = None;
            }
        }
    }

    fn on_key(&mut self, key: crossterm::event::KeyEvent) {
        if !self.modals.is_empty() {
            self.handle_modal_key(key);
            return;
        }
        if self.handle_tab_nav_key(key) {
            return;
        }
        let action = crate::tui::keymap::action_for(self.tab, key);
        if let Some(a) = action {
            self.dispatch_action(a);
        }
    }

    fn handle_tab_nav_key(&mut self, key: crossterm::event::KeyEvent) -> bool {
        match self.tab {
            Tab::Settings => {
                let (state, len) = match self.settings_ui.focus {
                    SettingsFocus::Boards => (
                        &mut self.settings_ui.board_state,
                        self.config.boards.len(),
                    ),
                    SettingsFocus::Clocks => (
                        &mut self.settings_ui.clock_state,
                        self.config.clocks.len(),
                    ),
                };
                scroll_list::handle_key(state, key, len)
            }
            Tab::Pending => {
                let len = self.pending.entries.len();
                if len == 0 {
                    return false;
                }
                use crossterm::event::KeyCode;
                let cur = self.pending_ui.table_state.selected().unwrap_or(0).min(len - 1);
                let new = match key.code {
                    KeyCode::Char('j') | KeyCode::Down => (cur + 1).min(len - 1),
                    KeyCode::Char('k') | KeyCode::Up => cur.saturating_sub(1),
                    KeyCode::Char('g') | KeyCode::Home => 0,
                    KeyCode::Char('G') | KeyCode::End => len - 1,
                    KeyCode::PageDown => (cur + 10).min(len - 1),
                    KeyCode::PageUp => cur.saturating_sub(10),
                    _ => return false,
                };
                self.pending_ui.table_state.select(Some(new));
                true
            }
            Tab::Projects => {
                let len = self.config.projects.len();
                scroll_list::handle_key(&mut self.projects.list_state, key, len)
            }
            _ => false,
        }
    }

    fn handle_modal_key(&mut self, key: crossterm::event::KeyEvent) {
        use crossterm::event::KeyCode;
        let Some(top) = self.modals.last() else { return; };
        match top {
            Modal::Confirm { on_confirm, .. } => match key.code {
                KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
                    let action = on_confirm.clone();
                    self.modals.pop();
                    self.run_confirm_action(action);
                }
                KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                    self.modals.pop();
                }
                _ => {}
            },
            Modal::EditPending { .. } => self.handle_edit_pending_key(key),
            Modal::AddProjectCode { .. } => self.handle_add_project_code_key(key),
            Modal::AddProjectKind { .. } => self.handle_add_project_kind_key(key),
            Modal::AddProjectPickBoard { .. } => self.handle_add_project_pick_board_key(key),
            Modal::AddProjectPickClock { .. } => self.handle_add_project_pick_clock_key(key),
            Modal::AddProjectSearchQuery { .. } => self.handle_add_project_search_query_key(key),
            Modal::AddProjectSearchResults { .. } => self.handle_add_project_search_results_key(key),
            Modal::PickIntegrationToRemove { .. } => self.handle_pick_integration_key(key),
            Modal::PickProject { .. } => self.handle_pick_project_key(key),
            Modal::SearchTaskQuery { .. } => self.handle_search_task_query_key(key),
            Modal::SearchTaskResults { .. } => self.handle_search_task_results_key(key),
            Modal::TimeStart { .. } => self.handle_time_start_key(key),
            Modal::TimeEnd { .. } => self.handle_time_end_key(key),
            Modal::StopTime { .. } => self.handle_stop_time_key(key),
            Modal::AuthBoardPlatform { .. } => self.handle_auth_board_platform_key(key),
            Modal::AuthClockPlatform { .. } => self.handle_auth_clock_platform_key(key),
            Modal::AuthLoginForm { .. } => self.handle_auth_login_form_key(key),
            Modal::DatePicker { .. } => self.handle_date_picker_key(key),
        }
    }

    fn handle_date_picker_key(&mut self, key: crossterm::event::KeyEvent) {
        use crossterm::event::KeyCode;
        let Some(Modal::DatePicker { state }) = self.modals.last_mut() else {
            return;
        };
        match key.code {
            KeyCode::Esc => {
                self.modals.pop();
            }
            KeyCode::Enter => match state.resolve() {
                Ok(date) => {
                    self.modals.pop();
                    let req_id = self.issue_req(InflightKind::SetActiveDate);
                    self.send(ServiceCommand::SetActiveDate { req_id, date });
                }
                Err(e) => {
                    state.error = Some(e);
                }
            },
            _ => {
                date_picker::handle_key(state, key);
            }
        }
    }

    fn handle_auth_board_platform_key(&mut self, key: crossterm::event::KeyEvent) {
        use crossterm::event::KeyCode;
        let Some(Modal::AuthBoardPlatform { state }) = self.modals.last_mut() else {
            return;
        };
        let len = 2usize;
        match key.code {
            KeyCode::Esc => {
                self.modals.pop();
            }
            KeyCode::Enter => {
                let sel = state.selected().unwrap_or(0);
                let platform = if sel == 0 {
                    crate::config::models::BoardPlatform::Jira
                } else {
                    crate::config::models::BoardPlatform::ClickUp
                };
                let draft = AuthDraft::board(platform);
                self.modals.pop();
                self.modals.push(Modal::AuthLoginForm {
                    draft,
                    focus: AuthFieldFocus::BaseUrl,
                    error: None,
                });
            }
            _ => {
                scroll_list::handle_key(state, key, len);
            }
        }
    }

    fn handle_auth_clock_platform_key(&mut self, key: crossterm::event::KeyEvent) {
        use crossterm::event::KeyCode;
        let Some(Modal::AuthClockPlatform { state }) = self.modals.last_mut() else {
            return;
        };
        let len = 2usize;
        match key.code {
            KeyCode::Esc => {
                self.modals.pop();
            }
            KeyCode::Enter => {
                let sel = state.selected().unwrap_or(0);
                let platform = if sel == 0 {
                    crate::config::models::ClockPlatform::Jira
                } else {
                    crate::config::models::ClockPlatform::Clockify
                };
                let draft = AuthDraft::clock(platform);
                self.modals.pop();
                self.modals.push(Modal::AuthLoginForm {
                    draft,
                    focus: AuthFieldFocus::BaseUrl,
                    error: None,
                });
            }
            _ => {
                scroll_list::handle_key(state, key, len);
            }
        }
    }

    fn handle_auth_login_form_key(&mut self, key: crossterm::event::KeyEvent) {
        use crossterm::event::KeyCode;
        let Some(Modal::AuthLoginForm { draft, focus, error }) = self.modals.last_mut() else {
            return;
        };
        match key.code {
            KeyCode::Esc => {
                self.modals.pop();
            }
            KeyCode::Tab => {
                *focus = next_visible_focus(draft, *focus, false);
            }
            KeyCode::BackTab => {
                *focus = next_visible_focus(draft, *focus, true);
            }
            KeyCode::Enter => {
                let submit_res = submit_auth_form(&self.config, draft);
                match submit_res {
                    Err(msg) => {
                        *error = Some(msg);
                    }
                    Ok((cmd, kind)) => {
                        self.modals.pop();
                        let req_id = self.issue_req(kind);
                        let cmd = with_req_id(cmd, req_id);
                        self.send(cmd);
                    }
                }
            }
            _ => {
                *error = None;
                let input = field_input_mut(draft, *focus);
                text_input::handle_key(input, key);
            }
        }
    }

    fn handle_pick_project_key(&mut self, key: crossterm::event::KeyEvent) {
        use crossterm::event::KeyCode;
        let Some(Modal::PickProject { state }) = self.modals.last_mut() else {
            return;
        };
        let len = self.config.projects.len();
        match key.code {
            KeyCode::Esc => {
                self.modals.pop();
            }
            KeyCode::Enter => {
                if len == 0 {
                    return;
                }
                let idx = state.selected().unwrap_or(0).min(len - 1);
                let code = self.config.projects[idx].code.clone();
                let cfg = &self.config;
                let board_id = match resolve_project(cfg, &code)
                    .and_then(|p| first_board_for_project(cfg, p))
                {
                    Ok(b) => b.id.clone(),
                    Err(e) => {
                        self.toast = Some(Toast::error(e.to_string()));
                        self.modals.pop();
                        return;
                    }
                };
                self.modals.pop();
                self.modals.push(Modal::SearchTaskQuery {
                    project_code: code,
                    board_id,
                    input: Input::default(),
                });
            }
            _ => {
                scroll_list::handle_key(state, key, len);
            }
        }
    }

    fn handle_search_task_query_key(&mut self, key: crossterm::event::KeyEvent) {
        use crossterm::event::KeyCode;
        let Some(Modal::SearchTaskQuery {
            project_code,
            board_id,
            input,
        }) = self.modals.last_mut()
        else {
            return;
        };
        match key.code {
            KeyCode::Esc => {
                self.modals.pop();
            }
            KeyCode::Enter => {
                let query = input.value().to_string();
                let project_code = project_code.clone();
                let board_id = board_id.clone();
                self.modals.pop();
                let req_id = self.issue_req(InflightKind::SearchTasks);
                let mut s = ListState::default();
                s.select(Some(0));
                self.modals.push(Modal::SearchTaskResults {
                    project_code: project_code.clone(),
                    board_id,
                    results: Vec::new(),
                    filtered: Vec::new(),
                    query: Input::default(),
                    state: s,
                    loading: true,
                    req_id,
                });
                self.send(ServiceCommand::SearchTasks {
                    req_id,
                    project_code,
                    query,
                });
            }
            _ => {
                text_input::handle_key(input, key);
            }
        }
    }

    fn handle_search_task_results_key(&mut self, key: crossterm::event::KeyEvent) {
        use crossterm::event::KeyCode;
        let Some(Modal::SearchTaskResults {
            project_code,
            board_id,
            results,
            filtered,
            query,
            state,
            req_id,
            ..
        }) = self.modals.last_mut()
        else {
            return;
        };
        match key.code {
            KeyCode::Esc => {
                let stale = *req_id;
                self.modals.pop();
                self.cancel_inflight(stale);
            }
            KeyCode::Enter => {
                if filtered.is_empty() {
                    return;
                }
                let sel = state.selected().unwrap_or(0).min(filtered.len() - 1);
                let task = results[filtered[sel]].clone();
                let draft = SessionDraft {
                    project_code: project_code.clone(),
                    board_id: board_id.clone(),
                    task,
                    start_hhmm: None,
                };
                let stale = *req_id;
                self.modals.pop();
                self.cancel_inflight(stale);
                self.modals.push(Modal::TimeStart {
                    draft,
                    input: TimeInputState::new(""),
                });
            }
            KeyCode::Up
            | KeyCode::Down
            | KeyCode::PageUp
            | KeyCode::PageDown
            | KeyCode::Home
            | KeyCode::End => {
                scroll_list::handle_key(state, key, filtered.len());
            }
            _ => {
                let handled = text_input::handle_key(query, key);
                if handled {
                    let labels: Vec<String> = results
                        .iter()
                        .map(|t| format!("{} {} {}", t.key, t.title, t.status))
                        .collect();
                    *filtered = fuzzy_select::recompute(&labels, query.value());
                    if state.selected().map(|i| i >= filtered.len()).unwrap_or(true) {
                        state.select(if filtered.is_empty() { None } else { Some(0) });
                    }
                }
            }
        }
    }

    fn handle_time_start_key(&mut self, key: crossterm::event::KeyEvent) {
        use crossterm::event::KeyCode;
        let Some(Modal::TimeStart { draft, input }) = self.modals.last_mut() else {
            return;
        };
        match key.code {
            KeyCode::Esc => {
                self.modals.pop();
            }
            KeyCode::Enter => {
                let raw = input.input.value().to_string();
                let hhmm = if raw.trim().is_empty() {
                    crate::utils::time::hhmm_from_now()
                } else {
                    match input.resolve() {
                        Ok(s) => s,
                        Err(e) => {
                            input.error = Some(e);
                            return;
                        }
                    }
                };
                let mut new_draft = draft.clone();
                new_draft.start_hhmm = Some(hhmm);
                self.modals.pop();
                self.modals.push(Modal::TimeEnd {
                    draft: new_draft,
                    input: TimeInputState::new(""),
                });
            }
            _ => {
                time_input::handle_key(input, key);
            }
        }
    }

    fn handle_time_end_key(&mut self, key: crossterm::event::KeyEvent) {
        use crossterm::event::KeyCode;
        let Some(Modal::TimeEnd { draft, input }) = self.modals.last_mut() else {
            return;
        };
        match key.code {
            KeyCode::Esc => {
                self.modals.pop();
            }
            KeyCode::Enter => {
                let raw = input.input.value().to_string();
                let end_hhmm = if raw.trim().is_empty() {
                    None
                } else {
                    match input.resolve() {
                        Ok(s) => Some(s),
                        Err(e) => {
                            input.error = Some(e);
                            return;
                        }
                    }
                };
                let draft = draft.clone();
                let start_hhmm = draft.start_hhmm.clone();
                self.modals.pop();

                let cfg = &self.config;
                let project = match resolve_project(cfg, &draft.project_code) {
                    Ok(p) => p.clone(),
                    Err(e) => {
                        self.toast = Some(Toast::error(e.to_string()));
                        return;
                    }
                };
                let session = build_session(
                    &project,
                    draft.board_id,
                    draft.task,
                    start_hhmm,
                    end_hhmm,
                    self.active_date,
                );
                let req_id = self.issue_req(InflightKind::StartSession);
                self.send(ServiceCommand::StartSession { req_id, session });
            }
            _ => {
                time_input::handle_key(input, key);
            }
        }
    }

    fn handle_stop_time_key(&mut self, key: crossterm::event::KeyEvent) {
        use crossterm::event::KeyCode;
        let Some(Modal::StopTime { input }) = self.modals.last_mut() else {
            return;
        };
        match key.code {
            KeyCode::Esc => {
                self.modals.pop();
            }
            KeyCode::Enter => {
                let stop_time = match input.resolve() {
                    Ok(s) => s,
                    Err(e) => {
                        input.error = Some(e);
                        return;
                    }
                };
                self.modals.pop();
                let req_id = self.issue_req(InflightKind::StopSession);
                self.send(ServiceCommand::StopSession { req_id, stop_time });
            }
            _ => {
                time_input::handle_key(input, key);
            }
        }
    }

    fn handle_edit_pending_key(&mut self, key: crossterm::event::KeyEvent) {
        use crossterm::event::KeyCode;
        let Some(Modal::EditPending { idx, fields, focus }) = self.modals.last_mut() else {
            return;
        };
        match key.code {
            KeyCode::Esc => {
                self.modals.pop();
            }
            KeyCode::Tab => {
                *focus = focus.next();
            }
            KeyCode::BackTab => {
                *focus = focus.prev();
            }
            KeyCode::Enter => {
                let hours_s = fields.hours.value().trim().to_string();
                let start_s = fields.start.value().trim().to_string();
                let end_s = fields.end.value().trim().to_string();
                let desc_s = fields.description.value().to_string();
                let idx = *idx;

                let hours = match hours_s.parse::<f32>() {
                    Ok(h) if h > 0.0 && h < 24.0 => h,
                    _ => {
                        fields.error = Some("Hours must be a positive number < 24".into());
                        return;
                    }
                };
                if !start_s.is_empty() && crate::utils::time::parse_hhmm(&start_s).is_err() {
                    fields.error = Some("Start must be HHMM".into());
                    return;
                }
                if !end_s.is_empty() && crate::utils::time::parse_hhmm(&end_s).is_err() {
                    fields.error = Some("End must be HHMM".into());
                    return;
                }

                let patch = PendingPatch {
                    hours: Some(hours),
                    start: Some(start_s),
                    end: Some(end_s),
                    description: Some(desc_s),
                };
                self.modals.pop();
                let req_id = self.issue_req(InflightKind::EditPending);
                self.send(ServiceCommand::EditPending {
                    req_id,
                    idx,
                    patch,
                });
            }
            _ => {
                let input = match focus {
                    EditPendingFocus::Hours => &mut fields.hours,
                    EditPendingFocus::Start => &mut fields.start,
                    EditPendingFocus::End => &mut fields.end,
                    EditPendingFocus::Description => &mut fields.description,
                };
                text_input::handle_key(input, key);
                fields.error = None;
            }
        }
    }

    fn handle_add_project_code_key(&mut self, key: crossterm::event::KeyEvent) {
        use crossterm::event::KeyCode;
        let Some(Modal::AddProjectCode { input }) = self.modals.last_mut() else {
            return;
        };
        match key.code {
            KeyCode::Esc => {
                self.modals.pop();
            }
            KeyCode::Enter => {
                let code = input.value().trim().to_string();
                if code.is_empty() {
                    self.toast = Some(Toast::warn("Project code required"));
                    return;
                }
                self.modals.pop();
                self.modals.push(Modal::AddProjectKind {
                    code,
                    state: {
                        let mut s = ListState::default();
                        s.select(Some(0));
                        s
                    },
                });
            }
            _ => {
                text_input::handle_key(input, key);
            }
        }
    }

    fn handle_add_project_kind_key(&mut self, key: crossterm::event::KeyEvent) {
        use crossterm::event::KeyCode;
        let Some(Modal::AddProjectKind { code, state }) = self.modals.last_mut() else {
            return;
        };
        let len = 2usize;
        match key.code {
            KeyCode::Esc => {
                self.modals.pop();
            }
            KeyCode::Enter => {
                let selected = state.selected().unwrap_or(0);
                let code = code.clone();
                self.modals.pop();
                if selected == 0 {
                    // Board flow
                    let boards = self.config.boards.clone();
                    if boards.is_empty() {
                        self.toast = Some(Toast::error(
                            "No board integrations. Run auth login first.",
                        ));
                        return;
                    }
                    let mut s = ListState::default();
                    s.select(Some(0));
                    self.modals.push(Modal::AddProjectPickBoard {
                        code,
                        options: boards,
                        state: s,
                    });
                } else {
                    // Clock flow
                    let options = clock_options_for(&self.config, &code);
                    if options.is_empty() {
                        self.toast = Some(Toast::error(
                            "No clock integrations available. Run auth login first.",
                        ));
                        return;
                    }
                    let mut s = ListState::default();
                    s.select(Some(0));
                    self.modals.push(Modal::AddProjectPickClock {
                        code,
                        options,
                        state: s,
                    });
                }
            }
            _ => {
                scroll_list::handle_key(state, key, len);
            }
        }
    }

    fn handle_add_project_pick_board_key(&mut self, key: crossterm::event::KeyEvent) {
        use crossterm::event::KeyCode;
        let Some(Modal::AddProjectPickBoard {
            code,
            options,
            state,
        }) = self.modals.last_mut()
        else {
            return;
        };
        let len = options.len();
        match key.code {
            KeyCode::Esc => {
                self.modals.pop();
            }
            KeyCode::Enter => {
                if len == 0 {
                    return;
                }
                let idx = state.selected().unwrap_or(0).min(len - 1);
                let board_id = options[idx].id.clone();
                let code = code.clone();
                self.modals.pop();
                self.modals.push(Modal::AddProjectSearchQuery {
                    code,
                    board_id,
                    input: Input::default(),
                });
            }
            _ => {
                scroll_list::handle_key(state, key, len);
            }
        }
    }

    fn handle_add_project_pick_clock_key(&mut self, key: crossterm::event::KeyEvent) {
        use crossterm::event::KeyCode;
        let Some(Modal::AddProjectPickClock {
            code,
            options,
            state,
        }) = self.modals.last_mut()
        else {
            return;
        };
        let len = options.len();
        match key.code {
            KeyCode::Esc => {
                self.modals.pop();
            }
            KeyCode::Enter => {
                if len == 0 {
                    return;
                }
                let idx = state.selected().unwrap_or(0).min(len - 1);
                let clock_choice = match &options[idx] {
                    ClockOption::Existing(id) => ClockChoice::Existing(id.clone()),
                    ClockOption::Derive { board_id } => {
                        ClockChoice::DeriveFromBoard(board_id.clone())
                    }
                };
                let code = code.clone();
                self.modals.pop();
                let req_id = self.issue_req(InflightKind::AddProject);
                self.send(ServiceCommand::AddProjectClock {
                    req_id,
                    code,
                    clock_choice,
                });
            }
            _ => {
                scroll_list::handle_key(state, key, len);
            }
        }
    }

    fn handle_add_project_search_query_key(&mut self, key: crossterm::event::KeyEvent) {
        use crossterm::event::KeyCode;
        let Some(Modal::AddProjectSearchQuery {
            code,
            board_id,
            input,
        }) = self.modals.last_mut()
        else {
            return;
        };
        match key.code {
            KeyCode::Esc => {
                self.modals.pop();
            }
            KeyCode::Enter => {
                let query = input.value().to_string();
                let code = code.clone();
                let board_id = board_id.clone();
                self.modals.pop();
                let req_id = self.issue_req(InflightKind::SearchProjects);
                let mut s = ListState::default();
                s.select(Some(0));
                self.modals.push(Modal::AddProjectSearchResults {
                    code,
                    board_id: board_id.clone(),
                    results: Vec::new(),
                    state: s,
                    loading: true,
                    req_id,
                });
                self.send(ServiceCommand::SearchProjects {
                    req_id,
                    board_id,
                    query,
                });
            }
            _ => {
                text_input::handle_key(input, key);
            }
        }
    }

    fn handle_add_project_search_results_key(&mut self, key: crossterm::event::KeyEvent) {
        use crossterm::event::KeyCode;
        let Some(Modal::AddProjectSearchResults {
            code,
            board_id,
            results,
            state,
            req_id,
            ..
        }) = self.modals.last_mut()
        else {
            return;
        };
        let len = results.len();
        match key.code {
            KeyCode::Esc => {
                let stale = *req_id;
                self.modals.pop();
                self.cancel_inflight(stale);
            }
            KeyCode::Enter => {
                if len == 0 {
                    return;
                }
                let idx = state.selected().unwrap_or(0).min(len - 1);
                let remote = results[idx].clone();
                let code = code.clone();
                let board_id = board_id.clone();
                self.modals.pop();
                let req_id = self.issue_req(InflightKind::AddProject);
                self.send(ServiceCommand::AddProjectBoard {
                    req_id,
                    code,
                    board_id,
                    remote,
                });
            }
            _ => {
                scroll_list::handle_key(state, key, len);
            }
        }
    }

    fn handle_pick_integration_key(&mut self, key: crossterm::event::KeyEvent) {
        use crossterm::event::KeyCode;
        let Some(Modal::PickIntegrationToRemove {
            code,
            options,
            state,
        }) = self.modals.last_mut()
        else {
            return;
        };
        let len = options.len();
        match key.code {
            KeyCode::Esc => {
                self.modals.pop();
            }
            KeyCode::Enter => {
                if len == 0 {
                    return;
                }
                let idx = state.selected().unwrap_or(0).min(len - 1);
                let picked = options[idx].clone();
                let code = code.clone();
                self.modals.pop();
                let (title, body) = build_unlink_confirm(&code, &picked, &self.pending);
                self.modals.push(Modal::Confirm {
                    title,
                    body,
                    on_confirm: ConfirmAction::RemoveProjectIntegration {
                        code,
                        kind: picked.kind,
                        id: picked.id,
                    },
                });
            }
            _ => {
                scroll_list::handle_key(state, key, len);
            }
        }
    }

    fn run_confirm_action(&mut self, action: ConfirmAction) {
        match action {
            ConfirmAction::Quit => self.should_quit = true,
            ConfirmAction::RemovePending(idx) => {
                let req_id = self.issue_req(InflightKind::RemovePending);
                self.send(ServiceCommand::RemovePending { req_id, idx });
            }
            ConfirmAction::PushAllPending => {
                let req_id = self.issue_req(InflightKind::PushPending);
                self.send(ServiceCommand::PushPending { req_id });
            }
            ConfirmAction::RemoveProjectIntegration { code, kind, id } => {
                let req_id = self.issue_req(InflightKind::RemoveProjectIntegration);
                self.send(ServiceCommand::RemoveProjectIntegration {
                    req_id,
                    code,
                    kind,
                    target_id: id,
                });
            }
            ConfirmAction::LogoutBoard(id) => {
                let req_id = self.issue_req(InflightKind::AuthLogout);
                self.send(ServiceCommand::AuthLogoutBoard { req_id, id });
            }
            ConfirmAction::LogoutClock(id) => {
                let req_id = self.issue_req(InflightKind::AuthLogout);
                self.send(ServiceCommand::AuthLogoutClock { req_id, id });
            }
        }
    }

    fn dispatch_action(&mut self, action: Action) {
        match action {
            Action::Quit => {
                if self.session.is_some() {
                    self.modals.push(Modal::Confirm {
                        title: "Quit klock?".into(),
                        body: "Discard the running session and quit?".into(),
                        on_confirm: ConfirmAction::Quit,
                    });
                } else {
                    self.should_quit = true;
                }
            }
            Action::NextTab => self.tab = self.tab.next(),
            Action::PrevTab => self.tab = self.tab.prev(),
            Action::JumpTab(t) => self.tab = t,
            Action::Refresh => self.kickoff_summary_fetch(),
            Action::StartSessionFlow => self.open_start_session_flow(),
            Action::StopSessionFlow => self.open_stop_session_flow(),
            Action::SetDateFlow => {
                self.modals.push(Modal::DatePicker {
                    state: DatePickerState::new(self.active_date),
                });
            }
            Action::PendingEdit => self.open_edit_pending(),
            Action::PendingRemove => self.open_confirm_remove_pending(),
            Action::PendingPushAll => self.open_confirm_push_all(),
            Action::ProjectsAdd => {
                self.modals.push(Modal::AddProjectCode {
                    input: Input::default(),
                });
            }
            Action::ProjectsRemove => self.open_pick_integration(),
            Action::SettingsAdd => self.open_settings_add(),
            Action::SettingsRemove => self.open_settings_logout(),
            Action::SettingsFocusBoards => {
                self.settings_ui.focus = SettingsFocus::Boards;
            }
            Action::SettingsFocusClocks => {
                self.settings_ui.focus = SettingsFocus::Clocks;
            }
        }
    }

    fn open_settings_add(&mut self) {
        let mut s = ListState::default();
        s.select(Some(0));
        match self.settings_ui.focus {
            SettingsFocus::Boards => {
                self.modals.push(Modal::AuthBoardPlatform { state: s });
            }
            SettingsFocus::Clocks => {
                self.modals.push(Modal::AuthClockPlatform { state: s });
            }
        }
    }

    fn open_settings_logout(&mut self) {
        match self.settings_ui.focus {
            SettingsFocus::Boards => {
                if self.config.boards.is_empty() {
                    return;
                }
                let idx = self
                    .settings_ui
                    .board_state
                    .selected()
                    .unwrap_or(0)
                    .min(self.config.boards.len() - 1);
                let board = &self.config.boards[idx];
                let id = board.id.clone();
                let email = board.email.clone();
                self.modals.push(Modal::Confirm {
                    title: format!("Logout board '{id}'?"),
                    body: format!("Remove {email} credentials from keyring and config."),
                    on_confirm: ConfirmAction::LogoutBoard(id),
                });
            }
            SettingsFocus::Clocks => {
                if self.config.clocks.is_empty() {
                    return;
                }
                let idx = self
                    .settings_ui
                    .clock_state
                    .selected()
                    .unwrap_or(0)
                    .min(self.config.clocks.len() - 1);
                let clock = &self.config.clocks[idx];
                let id = clock.id.clone();
                let email = clock.email.clone();
                self.modals.push(Modal::Confirm {
                    title: format!("Logout clock '{id}'?"),
                    body: format!("Remove {email} credentials from keyring and config."),
                    on_confirm: ConfirmAction::LogoutClock(id),
                });
            }
        }
    }

    fn open_start_session_flow(&mut self) {
        if self.session.is_some() {
            self.toast = Some(Toast::warn("Session already active — stop it first."));
            return;
        }
        if self.config.projects.is_empty() {
            self.toast = Some(Toast::error(
                "No projects configured. Add one on the Projects tab.",
            ));
            return;
        }
        let mut s = ListState::default();
        s.select(Some(0));
        self.modals.push(Modal::PickProject { state: s });
    }

    fn open_stop_session_flow(&mut self) {
        if self.session.is_none() {
            self.toast = Some(Toast::info("No active session."));
            return;
        }
        self.modals.push(Modal::StopTime {
            input: TimeInputState::new("now"),
        });
    }

    fn open_edit_pending(&mut self) {
        if self.pending.entries.is_empty() {
            return;
        }
        let idx_sel = self
            .pending_ui
            .table_state
            .selected()
            .unwrap_or(0)
            .min(self.pending.entries.len() - 1);
        let entry = &self.pending.entries[idx_sel];
        if !entry.pushed_clock_ids.is_empty() {
            self.toast = Some(Toast::warn(
                "Entry has partial pushes — remove it or finish push instead of editing.",
            ));
            return;
        }
        let fields = PendingEditDraft {
            hours: Input::new(format!("{:.2}", entry.hours)),
            start: Input::new(entry.start_time.clone().unwrap_or_default()),
            end: Input::new(entry.end_time.clone().unwrap_or_default()),
            description: Input::new(entry.description.clone()),
            error: None,
        };
        self.modals.push(Modal::EditPending {
            idx: entry.idx,
            fields,
            focus: EditPendingFocus::Hours,
        });
    }

    fn open_confirm_remove_pending(&mut self) {
        if self.pending.entries.is_empty() {
            return;
        }
        let idx_sel = self
            .pending_ui
            .table_state
            .selected()
            .unwrap_or(0)
            .min(self.pending.entries.len() - 1);
        let entry = &self.pending.entries[idx_sel];
        let title = format!("Remove pending #{}?", entry.idx);
        let body = format!(
            "{} — {:.2}h on {}",
            entry.task_key.clone().unwrap_or_else(|| "?".into()),
            entry.hours,
            entry.date
        );
        let entry_idx = entry.idx;
        self.modals.push(Modal::Confirm {
            title,
            body,
            on_confirm: ConfirmAction::RemovePending(entry_idx),
        });
    }

    fn open_confirm_push_all(&mut self) {
        if self.pending.entries.is_empty() {
            self.toast = Some(Toast::info("Nothing to push."));
            return;
        }
        let n = self.pending.entries.len();
        self.modals.push(Modal::Confirm {
            title: "Push all pending?".into(),
            body: format!("Push {} entry(ies) to their linked clocks?", n),
            on_confirm: ConfirmAction::PushAllPending,
        });
    }

    fn open_pick_integration(&mut self) {
        if self.config.projects.is_empty() {
            return;
        }
        let idx = self
            .projects
            .list_state
            .selected()
            .unwrap_or(0)
            .min(self.config.projects.len() - 1);
        let project = &self.config.projects[idx];
        let mut options: Vec<IntegrationRef> = Vec::new();
        for bid in &project.board_ids {
            let label = self
                .config
                .boards
                .iter()
                .find(|b| &b.id == bid)
                .map(|b| format!("{}  {}", b.id, b.email))
                .unwrap_or_else(|| format!("{} (missing)", bid));
            options.push(IntegrationRef {
                kind: IntegrationKind::Board,
                id: bid.clone(),
                label,
            });
        }
        for cid in &project.clock_ids {
            let label = self
                .config
                .clocks
                .iter()
                .find(|c| &c.id == cid)
                .map(|c| format!("{}  {}", c.id, c.email))
                .unwrap_or_else(|| format!("{} (missing)", cid));
            options.push(IntegrationRef {
                kind: IntegrationKind::Clock,
                id: cid.clone(),
                label,
            });
        }
        if options.is_empty() {
            self.toast = Some(Toast::info("No integrations linked to this project."));
            return;
        }
        let mut s = ListState::default();
        s.select(Some(0));
        self.modals.push(Modal::PickIntegrationToRemove {
            code: project.code.clone(),
            options,
            state: s,
        });
    }

    fn on_service(&mut self, msg: ServiceResult) {
        let req_id = service_req_id(&msg);
        if self.inflight.remove(&req_id).is_none() {
            return; // stale
        }
        match msg {
            ServiceResult::SummaryReady { date, rows, .. } => {
                if date == self.active_date {
                    self.dashboard.summary_rows = rows;
                    self.dashboard.summary_loading = false;
                    self.dashboard.last_summary_fetch = Some(std::time::Instant::now());
                }
            }
            ServiceResult::Error { message, .. } => {
                self.dashboard.summary_loading = false;
                self.toast = Some(Toast::error(message));
            }
            ServiceResult::SessionStarted { session, .. } => {
                self.session = Some(session);
                self.toast = Some(Toast::success("Session started"));
            }
            ServiceResult::SessionStopped {
                queued_idx,
                hours,
                pending,
                ..
            } => {
                self.pending = pending;
                self.session = None;
                self.toast = Some(Toast::success(format!(
                    "Queued #{queued_idx} — {hours:.2}h"
                )));
                clamp_table_state(&mut self.pending_ui.table_state, self.pending.entries.len());
            }
            ServiceResult::PendingChanged { store, .. } => {
                self.pending = store;
                self.toast = Some(Toast::success("Pending updated"));
                clamp_table_state(&mut self.pending_ui.table_state, self.pending.entries.len());
            }
            ServiceResult::PushReportReady { report, store, .. } => {
                self.pending = store;
                let msg = if !report.errors.is_empty() {
                    format!(
                        "{} pushed, {} partial, {} error(s)",
                        report.fully,
                        report.partial,
                        report.errors.len()
                    )
                } else if report.partial > 0 {
                    format!("{} fully pushed, {} partial", report.fully, report.partial)
                } else {
                    format!("{} pushed", report.fully)
                };
                if report.errors.is_empty() {
                    self.toast = Some(Toast::success(msg));
                } else {
                    self.toast = Some(Toast::warn(msg));
                }
                clamp_table_state(&mut self.pending_ui.table_state, self.pending.entries.len());
            }
            ServiceResult::ConfigChanged { config, .. } => {
                self.config = config;
                clamp_list_state(&mut self.projects.list_state, self.config.projects.len());
                clamp_list_state(&mut self.settings_ui.board_state, self.config.boards.len());
                clamp_list_state(&mut self.settings_ui.clock_state, self.config.clocks.len());
                // Close any pending AddProject flow since config just changed
                while matches!(
                    self.modals.last(),
                    Some(Modal::AddProjectCode { .. })
                        | Some(Modal::AddProjectKind { .. })
                        | Some(Modal::AddProjectPickBoard { .. })
                        | Some(Modal::AddProjectPickClock { .. })
                        | Some(Modal::AddProjectSearchQuery { .. })
                        | Some(Modal::AddProjectSearchResults { .. })
                        | Some(Modal::PickIntegrationToRemove { .. })
                ) {
                    self.modals.pop();
                }
                self.toast = Some(Toast::success("Config updated"));
            }
            ServiceResult::ActiveDateChanged { date, .. } => {
                self.active_date = date;
                self.kickoff_summary_fetch();
            }
            ServiceResult::ProjectsFound { req_id, results } => {
                if let Some(Modal::AddProjectSearchResults {
                    req_id: modal_req,
                    results: r,
                    loading,
                    state,
                    ..
                }) = self.modals.last_mut()
                {
                    if *modal_req == req_id {
                        *r = results;
                        *loading = false;
                        if state.selected().is_none() && !r.is_empty() {
                            state.select(Some(0));
                        }
                    }
                }
            }
            ServiceResult::TasksFound { req_id, results } => {
                if let Some(Modal::SearchTaskResults {
                    req_id: modal_req,
                    results: r,
                    filtered,
                    query,
                    state,
                    loading,
                    ..
                }) = self.modals.last_mut()
                {
                    if *modal_req == req_id {
                        *r = results;
                        *loading = false;
                        let labels: Vec<String> = r
                            .iter()
                            .map(|t| format!("{} {} {}", t.key, t.title, t.status))
                            .collect();
                        *filtered = fuzzy_select::recompute(&labels, query.value());
                        if state.selected().is_none() && !filtered.is_empty() {
                            state.select(Some(0));
                        }
                    }
                }
            }
        }
    }
}

fn field_input_mut(draft: &mut AuthDraft, field: AuthFieldFocus) -> &mut Input {
    match draft {
        AuthDraft::Board {
            base_url,
            email,
            team_id,
            secret,
            ..
        } => match field {
            AuthFieldFocus::BaseUrl => base_url,
            AuthFieldFocus::Email => email,
            AuthFieldFocus::TeamId => team_id,
            AuthFieldFocus::Secret => secret,
        },
        AuthDraft::Clock {
            base_url,
            email,
            secret,
            ..
        } => match field {
            AuthFieldFocus::BaseUrl => base_url,
            AuthFieldFocus::Email => email,
            AuthFieldFocus::Secret => secret,
            AuthFieldFocus::TeamId => base_url,
        },
    }
}

fn next_visible_focus(
    draft: &AuthDraft,
    current: AuthFieldFocus,
    reverse: bool,
) -> AuthFieldFocus {
    let visible = draft.visible_fields();
    let idx = visible.iter().position(|f| *f == current).unwrap_or(0);
    let new_idx = if reverse {
        (idx + visible.len() - 1) % visible.len()
    } else {
        (idx + 1) % visible.len()
    };
    visible[new_idx]
}

fn submit_auth_form(
    cfg: &AppConfig,
    draft: &AuthDraft,
) -> Result<(ServiceCommand, InflightKind), String> {
    match draft {
        AuthDraft::Board {
            platform,
            base_url,
            email,
            team_id,
            secret,
        } => {
            let base_url = base_url.value().trim().to_string();
            let email = email.value().trim().to_string();
            let team_id_v = team_id.value().trim().to_string();
            let secret = secret.value().to_string();
            if base_url.is_empty() {
                return Err("Base URL is required".into());
            }
            if email.is_empty() {
                return Err("Email is required".into());
            }
            if secret.is_empty() {
                return Err("API token is required".into());
            }
            let team_id = match platform {
                crate::config::models::BoardPlatform::ClickUp => {
                    if team_id_v.is_empty() {
                        return Err("Team ID is required for ClickUp".into());
                    }
                    Some(team_id_v)
                }
                _ => None,
            };
            let bdraft = crate::services::auth::BoardDraft {
                platform: platform.clone(),
                base_url,
                email,
                team_id,
            };
            if crate::services::auth::duplicate_board_exists(cfg, &bdraft) {
                return Err(
                    "An integration with this platform/URL/email already exists.".into(),
                );
            }
            Ok((
                ServiceCommand::AuthLoginBoard {
                    req_id: 0,
                    draft: bdraft,
                    secret,
                },
                InflightKind::AuthLogin,
            ))
        }
        AuthDraft::Clock {
            platform,
            base_url,
            email,
            secret,
        } => {
            let base_url = base_url.value().trim().to_string();
            let email = email.value().trim().to_string();
            let secret = secret.value().to_string();
            if base_url.is_empty() {
                return Err("Base URL is required".into());
            }
            if email.is_empty() {
                return Err("Email is required".into());
            }
            if secret.is_empty() {
                return Err("API token is required".into());
            }
            let cdraft = crate::services::auth::ClockDraft {
                platform: platform.clone(),
                base_url,
                email,
            };
            if crate::services::auth::duplicate_clock_exists(cfg, &cdraft) {
                return Err(
                    "An integration with this platform/URL/email already exists.".into(),
                );
            }
            Ok((
                ServiceCommand::AuthLoginClock {
                    req_id: 0,
                    draft: cdraft,
                    secret,
                },
                InflightKind::AuthLogin,
            ))
        }
    }
}

fn with_req_id(cmd: ServiceCommand, req_id: u64) -> ServiceCommand {
    match cmd {
        ServiceCommand::AuthLoginBoard { draft, secret, .. } => {
            ServiceCommand::AuthLoginBoard {
                req_id,
                draft,
                secret,
            }
        }
        ServiceCommand::AuthLoginClock { draft, secret, .. } => {
            ServiceCommand::AuthLoginClock {
                req_id,
                draft,
                secret,
            }
        }
        other => other,
    }
}

fn build_unlink_confirm(
    code: &str,
    picked: &IntegrationRef,
    pending: &PendingStore,
) -> (String, String) {
    let title = format!("Unlink {} from {}?", picked.kind.label(), code);
    let mut body = format!("Remove [{}] {} from project {code}.", picked.kind.label(), picked.label);
    if picked.kind == IntegrationKind::Clock {
        let affected = crate::services::projects::pending_affected_by_clock_unlink(
            pending, &picked.id,
        );
        if !affected.is_empty() {
            body.push_str(&format!(
                "\n\nWarning: {} pending entry(ies) reference this clock (#{}).",
                affected.len(),
                affected
                    .iter()
                    .map(|i| i.to_string())
                    .collect::<Vec<_>>()
                    .join(", #")
            ));
        }
    }
    (title, body)
}

fn service_req_id(msg: &ServiceResult) -> u64 {
    match msg {
        ServiceResult::SummaryReady { req_id, .. }
        | ServiceResult::SessionStarted { req_id, .. }
        | ServiceResult::SessionStopped { req_id, .. }
        | ServiceResult::PendingChanged { req_id, .. }
        | ServiceResult::PushReportReady { req_id, .. }
        | ServiceResult::ConfigChanged { req_id, .. }
        | ServiceResult::ActiveDateChanged { req_id, .. }
        | ServiceResult::ProjectsFound { req_id, .. }
        | ServiceResult::TasksFound { req_id, .. }
        | ServiceResult::Error { req_id, .. } => *req_id,
    }
}
