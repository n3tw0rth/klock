use crossterm::event::{KeyCode, KeyEvent};
use ratatui::widgets::ListState;

pub fn handle_key(state: &mut ListState, key: KeyEvent, len: usize) -> bool {
    if len == 0 {
        state.select(None);
        return false;
    }
    let cur = state.selected().unwrap_or(0).min(len - 1);
    let new = match key.code {
        KeyCode::Char('j') | KeyCode::Down => (cur + 1).min(len - 1),
        KeyCode::Char('k') | KeyCode::Up => cur.saturating_sub(1),
        KeyCode::Char('g') | KeyCode::Home => 0,
        KeyCode::Char('G') | KeyCode::End => len - 1,
        KeyCode::PageDown => (cur + 10).min(len - 1),
        KeyCode::PageUp => cur.saturating_sub(10),
        _ => return false,
    };
    state.select(Some(new));
    true
}
