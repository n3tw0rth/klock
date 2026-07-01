use chrono::Local;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table};
use ratatui::Frame;

use crate::tui::app::App;
use crate::utils::time::format_elapsed;

pub fn draw(frame: &mut Frame, app: &mut App, area: Rect) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(6),
            Constraint::Min(5),
            Constraint::Length(1),
        ])
        .split(area);

    draw_session_card(frame, app, layout[0]);
    draw_summary_table(frame, app, layout[1]);
    draw_hint_row(frame, layout[2]);
}

fn draw_session_card(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Active session ")
        .border_style(Style::default().fg(Color::Green));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    match &app.session {
        None => {
            let para = Paragraph::new(Line::from(Span::styled(
                "No active session — press [s] to start one",
                Style::default().fg(Color::DarkGray),
            )))
            .alignment(Alignment::Center);
            frame.render_widget(para, inner);
        }
        Some(s) => {
            let elapsed = Local::now().signed_duration_since(s.started_at);
            let key = s
                .task_key
                .clone()
                .unwrap_or_else(|| s.task_id.clone());
            let title_line = Line::from(vec![
                Span::styled(
                    format!("[{key}] "),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(s.task_title.clone()),
            ]);
            let meta = Line::from(vec![
                Span::styled("project: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    s.project_code.clone(),
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::raw("   "),
                Span::styled("date: ", Style::default().fg(Color::DarkGray)),
                Span::raw(s.active_date.format("%Y-%m-%d").to_string()),
                Span::raw("   "),
                Span::styled("elapsed: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format_elapsed(elapsed),
                    Style::default().fg(Color::Yellow),
                ),
            ]);
            let start_line = if let Some(start) = &s.start_time_override {
                Line::from(vec![
                    Span::styled("start: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(start.clone(), Style::default().fg(Color::Yellow)),
                ])
            } else {
                Line::from("")
            };
            let para = Paragraph::new(vec![title_line, meta, start_line]);
            frame.render_widget(para, inner);
        }
    }
}

fn draw_summary_table(frame: &mut Frame, app: &App, area: Rect) {
    let title = format!(
        " Summary — {} ",
        app.active_date.format("%Y-%m-%d")
    );
    let mut block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(Style::default().fg(Color::Blue));

    if app.dashboard.summary_loading {
        block = block.title(format!(
            " Summary — {} {} loading… ",
            app.active_date.format("%Y-%m-%d"),
            app.spinner.glyph()
        ));
    }
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if app.dashboard.summary_rows.is_empty() && !app.dashboard.summary_loading {
        let para = Paragraph::new(Line::from(Span::styled(
            "No projects or clocks configured.",
            Style::default().fg(Color::DarkGray),
        )))
        .alignment(Alignment::Center);
        frame.render_widget(para, inner);
        return;
    }

    let header = Row::new(vec![
        Cell::from("Project").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Clock").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Hours").style(Style::default().add_modifier(Modifier::BOLD)),
    ]);

    let mut total = 0.0f32;
    let rows: Vec<Row> = app
        .dashboard
        .summary_rows
        .iter()
        .map(|r| {
            let hours_cell = match &r.hours {
                Ok(h) => {
                    total += *h;
                    Cell::from(format!("{:.1}h", h))
                        .style(Style::default().fg(Color::Yellow))
                }
                Err(e) => Cell::from(e.clone()).style(Style::default().fg(Color::Red)),
            };
            let project = if r.project_name.is_empty() {
                r.project_code.clone()
            } else {
                format!("{} — {}", r.project_code, r.project_name)
            };
            Row::new(vec![
                Cell::from(project),
                Cell::from(r.clock_id.clone()),
                hours_cell,
            ])
        })
        .collect();

    let widths = [
        Constraint::Percentage(50),
        Constraint::Percentage(30),
        Constraint::Percentage(20),
    ];
    let table = Table::new(rows, widths).header(header);

    let table_area = Rect {
        x: inner.x,
        y: inner.y,
        width: inner.width,
        height: inner.height.saturating_sub(2),
    };
    frame.render_widget(table, table_area);

    let total_line = Line::from(vec![
        Span::styled("Total: ", Style::default().add_modifier(Modifier::BOLD)),
        Span::styled(
            format!("{:.1}h", total),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
    ]);
    let total_area = Rect {
        x: inner.x,
        y: inner.y + inner.height.saturating_sub(1),
        width: inner.width,
        height: 1,
    };
    let total_para = Paragraph::new(total_line).alignment(Alignment::Right);
    frame.render_widget(total_para, total_area);
}

fn draw_hint_row(frame: &mut Frame, area: Rect) {
    let line = Line::from(Span::styled(
        "[s] start  [x] stop  [S] date  [r] refresh",
        Style::default().fg(Color::DarkGray),
    ));
    let para = Paragraph::new(line).alignment(Alignment::Center);
    frame.render_widget(para, area);
}

pub fn keybindings_hint() -> Vec<(&'static str, &'static str)> {
    vec![
        ("s", "start"),
        ("x", "stop"),
        ("S", "date"),
        ("r", "refresh"),
        ("Tab", "next"),
        ("q", "quit"),
    ]
}
