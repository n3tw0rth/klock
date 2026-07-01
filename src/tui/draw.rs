use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Tabs};
use ratatui::Frame;

use crate::tui::app::App;
use crate::tui::state::Tab;
use crate::tui::tabs;
use crate::tui::widgets::{overlay, status_bar, toast};

pub fn draw(frame: &mut Frame, app: &mut App) {
    let area = frame.area();
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(1),
        ])
        .split(area);

    draw_title_bar(frame, app, layout[0]);
    tabs::draw_active(frame, app, layout[1]);

    let hints = if app.modals.is_empty() {
        tabs::keybindings_hint(app)
    } else {
        vec![("Enter", "confirm"), ("Esc", "cancel"), ("Tab", "next field")]
    };
    let hint_refs: Vec<(&str, &str)> = hints.iter().map(|(k, v)| (*k, *v)).collect();
    status_bar::draw(frame, layout[2], &hint_refs);

    if !app.modals.is_empty() {
        overlay::draw(frame, app, area);
    }
    if let Some(t) = &app.toast {
        if !t.is_expired() {
            toast::draw(frame, t, area);
        }
    }
}

fn draw_title_bar(frame: &mut Frame, app: &App, area: Rect) {
    let titles: Vec<Line> = Tab::ALL
        .iter()
        .map(|t| {
            Line::from(Span::styled(
                format!(" {} ", t.title()),
                Style::default().add_modifier(Modifier::BOLD),
            ))
        })
        .collect();
    let block = Block::default()
        .borders(Borders::BOTTOM)
        .title(Line::from(vec![
            Span::styled(
                " klock ",
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  modern time tracker"),
        ]));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let tabs_widget = Tabs::new(titles)
        .select(app.tab.index())
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .divider("");
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(30)])
        .split(inner);
    frame.render_widget(tabs_widget, layout[0]);

    let right = Paragraph::new(Line::from(vec![
        Span::styled("date: ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            app.active_date.format("%Y-%m-%d").to_string(),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
    ]))
    .alignment(ratatui::layout::Alignment::Right);
    frame.render_widget(right, layout[1]);
}
