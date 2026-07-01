use std::time::Instant;

use ratatui::widgets::{ListState, TableState};

use crate::services::summary::SummaryRow;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Dashboard,
    Projects,
    Pending,
    Settings,
}

impl Tab {
    pub const ALL: [Tab; 4] = [Tab::Dashboard, Tab::Projects, Tab::Pending, Tab::Settings];

    pub fn title(self) -> &'static str {
        match self {
            Tab::Dashboard => "Dashboard",
            Tab::Projects => "Projects",
            Tab::Pending => "Pending",
            Tab::Settings => "Settings",
        }
    }

    pub fn index(self) -> usize {
        Self::ALL.iter().position(|t| *t == self).unwrap()
    }

    pub fn next(self) -> Tab {
        let i = (self.index() + 1) % Self::ALL.len();
        Self::ALL[i]
    }

    pub fn prev(self) -> Tab {
        let i = (self.index() + Self::ALL.len() - 1) % Self::ALL.len();
        Self::ALL[i]
    }
}

#[derive(Debug, Default)]
pub struct DashboardState {
    pub summary_rows: Vec<SummaryRow>,
    pub summary_loading: bool,
    pub last_summary_fetch: Option<Instant>,
}

#[derive(Debug, Default)]
pub struct ProjectsState {
    pub list_state: ListState,
}

#[derive(Debug, Default)]
pub struct PendingUiState {
    pub table_state: TableState,
}

pub fn clamp_table_state(state: &mut TableState, len: usize) {
    if len == 0 {
        state.select(None);
        return;
    }
    match state.selected() {
        Some(i) if i >= len => state.select(Some(len - 1)),
        None => state.select(Some(0)),
        _ => {}
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsFocus {
    Boards,
    Clocks,
}

impl Default for SettingsFocus {
    fn default() -> Self {
        SettingsFocus::Boards
    }
}

#[derive(Debug, Default)]
pub struct SettingsState {
    pub focus: SettingsFocus,
    pub board_state: ListState,
    pub clock_state: ListState,
}

pub fn clamp_list_state(state: &mut ListState, len: usize) {
    if len == 0 {
        state.select(None);
        return;
    }
    match state.selected() {
        Some(i) if i >= len => state.select(Some(len - 1)),
        None => state.select(Some(0)),
        _ => {}
    }
}
