use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::tui::state::Tab;

#[derive(Debug, Clone, Copy)]
pub enum Action {
    Quit,
    NextTab,
    PrevTab,
    JumpTab(Tab),
    Refresh,
    StartSessionFlow,
    StopSessionFlow,
    SetDateFlow,
    PendingEdit,
    PendingRemove,
    PendingPushAll,
    ProjectsAdd,
    ProjectsRemove,
    SettingsAdd,
    SettingsRemove,
    SettingsFocusBoards,
    SettingsFocusClocks,
}

pub fn action_for(tab: Tab, key: KeyEvent) -> Option<Action> {
    if let Some(global) = global(key) {
        return Some(global);
    }
    match tab {
        Tab::Dashboard => dashboard(key),
        Tab::Projects => projects(key),
        Tab::Pending => pending(key),
        Tab::Settings => settings(key),
    }
}

fn global(key: KeyEvent) -> Option<Action> {
    match key.code {
        KeyCode::Char('q') => Some(Action::Quit),
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Some(Action::Quit)
        }
        KeyCode::Tab => Some(Action::NextTab),
        KeyCode::BackTab => Some(Action::PrevTab),
        KeyCode::Char('1') => Some(Action::JumpTab(Tab::Dashboard)),
        KeyCode::Char('2') => Some(Action::JumpTab(Tab::Projects)),
        KeyCode::Char('3') => Some(Action::JumpTab(Tab::Pending)),
        KeyCode::Char('4') => Some(Action::JumpTab(Tab::Settings)),
        _ => None,
    }
}

fn dashboard(key: KeyEvent) -> Option<Action> {
    match key.code {
        KeyCode::Char('r') => Some(Action::Refresh),
        KeyCode::Char('s') => Some(Action::StartSessionFlow),
        KeyCode::Char('x') => Some(Action::StopSessionFlow),
        KeyCode::Char('S') => Some(Action::SetDateFlow),
        _ => None,
    }
}

fn pending(key: KeyEvent) -> Option<Action> {
    match key.code {
        KeyCode::Char('e') | KeyCode::Enter => Some(Action::PendingEdit),
        KeyCode::Char('d') => Some(Action::PendingRemove),
        KeyCode::Char('p') => Some(Action::PendingPushAll),
        _ => None,
    }
}

fn projects(key: KeyEvent) -> Option<Action> {
    match key.code {
        KeyCode::Char('a') => Some(Action::ProjectsAdd),
        KeyCode::Char('d') => Some(Action::ProjectsRemove),
        _ => None,
    }
}

fn settings(key: KeyEvent) -> Option<Action> {
    match key.code {
        KeyCode::Char('a') => Some(Action::SettingsAdd),
        KeyCode::Char('d') => Some(Action::SettingsRemove),
        KeyCode::Char('h') | KeyCode::Left => Some(Action::SettingsFocusBoards),
        KeyCode::Char('l') | KeyCode::Right => Some(Action::SettingsFocusClocks),
        _ => None,
    }
}
