use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui::Frame;

use super::overlay::centered_rect;

pub fn draw(frame: &mut Frame, area: Rect, title: &str, body: &str) {
    let rect = centered_rect(50, 30, area);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {title} "))
        .border_style(Style::default().fg(Color::Yellow));
    let inner = block.inner(rect);
    frame.render_widget(Clear, rect);
    frame.render_widget(block, rect);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(inner);

    let para = Paragraph::new(body.to_string())
        .wrap(Wrap { trim: true })
        .alignment(Alignment::Center);
    frame.render_widget(para, layout[0]);

    let hint = Paragraph::new(Line::from(vec![
        Span::styled(
            " y ",
            Style::default()
                .bg(Color::Green)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(
            " n ",
            Style::default()
                .bg(Color::Red)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled("Esc cancel", Style::default().fg(Color::Gray)),
    ]))
    .alignment(Alignment::Center);
    frame.render_widget(hint, layout[1]);
}
