use ratatui::layout::Rect;
use ratatui::Frame;

use crate::tui::app::App;
use crate::tui::state::Tab;

pub mod dashboard;
pub mod pending;
pub mod projects;
pub mod settings;

pub fn draw_active(frame: &mut Frame, app: &mut App, area: Rect) {
    match app.tab {
        Tab::Dashboard => dashboard::draw(frame, app, area),
        Tab::Projects => projects::draw(frame, app, area),
        Tab::Pending => pending::draw(frame, app, area),
        Tab::Settings => settings::draw(frame, app, area),
    }
}

pub fn keybindings_hint(app: &App) -> Vec<(&'static str, &'static str)> {
    match app.tab {
        Tab::Dashboard => dashboard::keybindings_hint(),
        Tab::Projects => projects::keybindings_hint(),
        Tab::Pending => pending::keybindings_hint(),
        Tab::Settings => settings::keybindings_hint(),
    }
}
