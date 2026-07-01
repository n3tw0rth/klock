use ratatui::widgets::ListState;
use tui_input::Input;

use crate::boards::{RemoteProject, RemoteTask};
use crate::config::models::{BoardConfig, BoardPlatform, ClockPlatform};
use crate::services::projects::{ClockOption, IntegrationKind};
use crate::tui::widgets::date_picker::DatePickerState;
use crate::tui::widgets::time_input::TimeInputState;

#[derive(Debug, Clone)]
pub enum Modal {
    Confirm {
        title: String,
        body: String,
        on_confirm: ConfirmAction,
    },
    EditPending {
        idx: u32,
        fields: PendingEditDraft,
        focus: EditPendingFocus,
    },
    AddProjectCode {
        input: Input,
    },
    AddProjectKind {
        code: String,
        state: ListState,
    },
    AddProjectPickBoard {
        code: String,
        options: Vec<BoardConfig>,
        state: ListState,
    },
    AddProjectPickClock {
        code: String,
        options: Vec<ClockOption>,
        state: ListState,
    },
    AddProjectSearchQuery {
        code: String,
        board_id: String,
        input: Input,
    },
    AddProjectSearchResults {
        code: String,
        board_id: String,
        results: Vec<RemoteProject>,
        state: ListState,
        loading: bool,
        req_id: u64,
    },
    PickIntegrationToRemove {
        code: String,
        options: Vec<IntegrationRef>,
        state: ListState,
    },
    PickProject {
        state: ListState,
    },
    SearchTaskQuery {
        project_code: String,
        board_id: String,
        input: Input,
    },
    SearchTaskResults {
        project_code: String,
        board_id: String,
        results: Vec<RemoteTask>,
        filtered: Vec<usize>,
        query: Input,
        state: ListState,
        loading: bool,
        req_id: u64,
    },
    TimeStart {
        draft: SessionDraft,
        input: TimeInputState,
    },
    TimeEnd {
        draft: SessionDraft,
        input: TimeInputState,
    },
    StopTime {
        input: TimeInputState,
    },
    AuthBoardPlatform {
        state: ListState,
    },
    AuthClockPlatform {
        state: ListState,
    },
    AuthLoginForm {
        draft: AuthDraft,
        focus: AuthFieldFocus,
        error: Option<String>,
    },
    DatePicker {
        state: DatePickerState,
    },
}

#[derive(Debug, Clone)]
pub enum AuthDraft {
    Board {
        platform: BoardPlatform,
        base_url: Input,
        email: Input,
        team_id: Input,
        secret: Input,
    },
    Clock {
        platform: ClockPlatform,
        base_url: Input,
        email: Input,
        secret: Input,
    },
}

impl AuthDraft {
    pub fn board(platform: BoardPlatform) -> Self {
        let default_url = match platform {
            BoardPlatform::ClickUp => "https://api.clickup.com",
            BoardPlatform::Jira => "",
        };
        AuthDraft::Board {
            platform,
            base_url: Input::new(default_url.to_string()),
            email: Input::default(),
            team_id: Input::default(),
            secret: Input::default(),
        }
    }

    pub fn clock(platform: ClockPlatform) -> Self {
        let default_url = match platform {
            ClockPlatform::Clockify => "https://api.clockify.me",
            ClockPlatform::Jira => "",
        };
        AuthDraft::Clock {
            platform,
            base_url: Input::new(default_url.to_string()),
            email: Input::default(),
            secret: Input::default(),
        }
    }

    pub fn visible_fields(&self) -> Vec<AuthFieldFocus> {
        match self {
            AuthDraft::Board { platform, .. } => match platform {
                BoardPlatform::Jira => vec![
                    AuthFieldFocus::BaseUrl,
                    AuthFieldFocus::Email,
                    AuthFieldFocus::Secret,
                ],
                BoardPlatform::ClickUp => vec![
                    AuthFieldFocus::BaseUrl,
                    AuthFieldFocus::TeamId,
                    AuthFieldFocus::Email,
                    AuthFieldFocus::Secret,
                ],
            },
            AuthDraft::Clock { .. } => vec![
                AuthFieldFocus::BaseUrl,
                AuthFieldFocus::Email,
                AuthFieldFocus::Secret,
            ],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthFieldFocus {
    BaseUrl,
    TeamId,
    Email,
    Secret,
}

impl AuthFieldFocus {
    pub fn label(self) -> &'static str {
        match self {
            AuthFieldFocus::BaseUrl => "Base URL",
            AuthFieldFocus::TeamId => "Team ID",
            AuthFieldFocus::Email => "Email",
            AuthFieldFocus::Secret => "API Token",
        }
    }
}

#[derive(Debug, Clone)]
pub struct SessionDraft {
    pub project_code: String,
    pub board_id: String,
    pub task: RemoteTask,
    pub start_hhmm: Option<String>,
}

#[derive(Debug, Clone)]
pub enum ConfirmAction {
    Quit,
    RemovePending(u32),
    LogoutBoard(String),
    LogoutClock(String),
    RemoveProjectIntegration {
        code: String,
        kind: IntegrationKind,
        id: String,
    },
    PushAllPending,
}

#[derive(Debug, Clone)]
pub struct PendingEditDraft {
    pub hours: Input,
    pub start: Input,
    pub end: Input,
    pub description: Input,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditPendingFocus {
    Hours,
    Start,
    End,
    Description,
}

impl EditPendingFocus {
    pub fn next(self) -> Self {
        match self {
            EditPendingFocus::Hours => EditPendingFocus::Start,
            EditPendingFocus::Start => EditPendingFocus::End,
            EditPendingFocus::End => EditPendingFocus::Description,
            EditPendingFocus::Description => EditPendingFocus::Hours,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            EditPendingFocus::Hours => EditPendingFocus::Description,
            EditPendingFocus::Start => EditPendingFocus::Hours,
            EditPendingFocus::End => EditPendingFocus::Start,
            EditPendingFocus::Description => EditPendingFocus::End,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            EditPendingFocus::Hours => "Hours",
            EditPendingFocus::Start => "Start (HHMM)",
            EditPendingFocus::End => "End (HHMM)",
            EditPendingFocus::Description => "Description",
        }
    }
}

#[derive(Debug, Clone)]
pub struct IntegrationRef {
    pub kind: IntegrationKind,
    pub id: String,
    pub label: String,
}
