use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use ratatui::Frame;

use crate::config::models::{BoardConfig, BoardPlatform, ClockConfig, ClockPlatform};
use crate::tui::app::App;
use crate::tui::state::{clamp_list_state, SettingsFocus};

pub fn draw(frame: &mut Frame, app: &mut App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Settings ")
        .border_style(Style::default().fg(Color::Cyan));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    clamp_list_state(&mut app.settings_ui.board_state, app.config.boards.len());
    clamp_list_state(&mut app.settings_ui.clock_state, app.config.clocks.len());

    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(inner);

    let boards_focused = app.settings_ui.focus == SettingsFocus::Boards;
    let clocks_focused = app.settings_ui.focus == SettingsFocus::Clocks;

    draw_pane(
        frame,
        layout[0],
        "Boards",
        &board_labels(&app.config.boards),
        &mut app.settings_ui.board_state,
        boards_focused,
    );
    draw_pane(
        frame,
        layout[1],
        "Clocks",
        &clock_labels(&app.config.clocks),
        &mut app.settings_ui.clock_state,
        clocks_focused,
    );
}

fn draw_pane(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    items: &[String],
    state: &mut ratatui::widgets::ListState,
    focused: bool,
) {
    let border_style = if focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {title} "))
        .border_style(border_style);

    if items.is_empty() {
        let inner = block.inner(area);
        frame.render_widget(block, area);
        let msg = Paragraph::new(Line::from(Span::styled(
            "(none)",
            Style::default().fg(Color::DarkGray),
        )))
        .alignment(Alignment::Center);
        frame.render_widget(msg, inner);
        return;
    }

    let list_items: Vec<ListItem> = items
        .iter()
        .map(|s| ListItem::new(s.clone()))
        .collect();
    let list = List::new(list_items)
        .block(block)
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::REVERSED)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▸ ");
    frame.render_stateful_widget(list, area, state);
}

fn board_labels(boards: &[BoardConfig]) -> Vec<String> {
    boards
        .iter()
        .map(|b| {
            let platform = match b.platform {
                BoardPlatform::Jira => "jira",
                BoardPlatform::ClickUp => "clickup",
            };
            format!("[{}] {}  {}  ({})", platform, b.id, b.email, b.base_url)
        })
        .collect()
}

fn clock_labels(clocks: &[ClockConfig]) -> Vec<String> {
    clocks
        .iter()
        .map(|c| {
            let platform = match c.platform {
                ClockPlatform::Jira => "jira",
                ClockPlatform::Clockify => "clockify",
            };
            format!("[{}] {}  {}  ({})", platform, c.id, c.email, c.base_url)
        })
        .collect()
}

pub fn keybindings_hint() -> Vec<(&'static str, &'static str)> {
    vec![
        ("j/k", "nav"),
        ("Tab", "toggle pane"),
        ("a", "add"),
        ("d", "logout"),
        ("q", "quit"),
    ]
}
