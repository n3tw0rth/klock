use ratatui::layout::{Alignment, Constraint, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table};
use ratatui::Frame;

use crate::config::models::PendingEntry;
use crate::tui::app::App;
use crate::tui::state::clamp_table_state;

pub fn draw(frame: &mut Frame, app: &mut App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Pending ")
        .border_style(Style::default().fg(Color::Blue));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if app.pending.entries.is_empty() {
        let msg = Paragraph::new(Line::from(Span::styled(
            "No pending entries. Start a session to queue one.",
            Style::default().fg(Color::DarkGray),
        )))
        .alignment(Alignment::Center);
        frame.render_widget(msg, inner);
        return;
    }

    clamp_table_state(&mut app.pending_ui.table_state, app.pending.entries.len());

    let header = Row::new(vec![
        Cell::from("#"),
        Cell::from("Task"),
        Cell::from("Title"),
        Cell::from("Date"),
        Cell::from("Start→End"),
        Cell::from("Hours"),
        Cell::from("Clocks"),
    ])
    .style(
        Style::default()
            .fg(Color::DarkGray)
            .add_modifier(Modifier::BOLD),
    );

    let rows: Vec<Row> = app
        .pending
        .entries
        .iter()
        .map(|e| build_row(e))
        .collect();

    let widths = [
        Constraint::Length(5),
        Constraint::Length(14),
        Constraint::Min(20),
        Constraint::Length(12),
        Constraint::Length(11),
        Constraint::Length(6),
        Constraint::Length(18),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .highlight_style(
            Style::default()
                .bg(Color::Blue)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▸ ");

    frame.render_stateful_widget(table, inner, &mut app.pending_ui.table_state);
}

fn build_row(entry: &PendingEntry) -> Row<'static> {
    let task_key = entry.task_key.clone().unwrap_or_else(|| "-".into());
    let title = truncate(&entry.task_title, 40);
    let date = entry.date.format("%Y-%m-%d").to_string();
    let times = format!(
        "{}→{}",
        entry.start_time.as_deref().unwrap_or("----"),
        entry.end_time.as_deref().unwrap_or("----"),
    );
    let hours = format!("{:.2}", entry.hours);

    let pushed = entry.pushed_clock_ids.len();
    let total = entry.clock_ids.len();
    let clocks_style = if pushed == total && total > 0 {
        Style::default().fg(Color::Green)
    } else if pushed > 0 {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let clocks_txt = format!("{}/{} pushed", pushed, total);

    Row::new(vec![
        Cell::from(format!("#{}", entry.idx)),
        Cell::from(task_key),
        Cell::from(title),
        Cell::from(date),
        Cell::from(times),
        Cell::from(hours),
        Cell::from(clocks_txt).style(clocks_style),
    ])
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let mut out: String = s.chars().take(max.saturating_sub(1)).collect();
        out.push('…');
        out
    }
}

pub fn keybindings_hint() -> Vec<(&'static str, &'static str)> {
    vec![
        ("j/k", "nav"),
        ("e", "edit"),
        ("d", "remove"),
        ("p", "push all"),
        ("Tab", "next tab"),
        ("q", "quit"),
    ]
}
