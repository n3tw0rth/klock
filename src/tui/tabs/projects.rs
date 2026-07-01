use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Wrap};
use ratatui::Frame;

use crate::config::models::ProjectConfig;
use crate::tui::app::App;
use crate::tui::state::clamp_list_state;

pub fn draw(frame: &mut Frame, app: &mut App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Projects ")
        .border_style(Style::default().fg(Color::Magenta));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if app.config.projects.is_empty() {
        let msg = Paragraph::new(Line::from(Span::styled(
            "No projects linked. Press `a` to add one.",
            Style::default().fg(Color::DarkGray),
        )))
        .alignment(Alignment::Center);
        frame.render_widget(msg, inner);
        return;
    }

    clamp_list_state(&mut app.projects.list_state, app.config.projects.len());

    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(24), Constraint::Min(20)])
        .split(inner);

    let items: Vec<ListItem> = app
        .config
        .projects
        .iter()
        .map(|p| ListItem::new(p.code.clone()))
        .collect();
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::RIGHT)
                .title(" Code "),
        )
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::REVERSED)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▸ ");
    frame.render_stateful_widget(list, layout[0], &mut app.projects.list_state);

    let selected_idx = app.projects.list_state.selected().unwrap_or(0);
    let project = &app.config.projects[selected_idx];
    draw_detail(frame, layout[1], project, app);
}

fn draw_detail(frame: &mut Frame, area: Rect, project: &ProjectConfig, app: &App) {
    let inner_layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(2),
            Constraint::Length(2),
            Constraint::Length(1),
            Constraint::Length(5),
            Constraint::Length(1),
            Constraint::Length(5),
            Constraint::Length(1),
            Constraint::Min(1),
        ])
        .split(area);

    let title = Paragraph::new(Line::from(vec![
        Span::styled(
            project.code.clone(),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("  ({})", project.platform_project_name),
            Style::default().fg(Color::Gray),
        ),
    ]));
    frame.render_widget(title, inner_layout[0]);

    let platform_id = Paragraph::new(Line::from(vec![
        Span::styled("platform id: ", Style::default().fg(Color::DarkGray)),
        Span::raw(project.platform_project_id.clone()),
    ]));
    frame.render_widget(platform_id, inner_layout[1]);

    let boards_hdr = Paragraph::new(Line::from(Span::styled(
        "Boards",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )));
    frame.render_widget(boards_hdr, inner_layout[2]);

    let boards_body: Vec<Line> = if project.board_ids.is_empty() {
        vec![Line::from(Span::styled(
            "  (none)",
            Style::default().fg(Color::DarkGray),
        ))]
    } else {
        project
            .board_ids
            .iter()
            .map(|id| {
                let matched = app.config.boards.iter().find(|b| &b.id == id);
                match matched {
                    Some(b) => Line::from(format!("  {}  {}  ({})", b.id, b.email, b.base_url)),
                    None => Line::from(format!("  {}  (missing)", id)),
                }
            })
            .collect()
    };
    let boards_para = Paragraph::new(boards_body).wrap(Wrap { trim: false });
    frame.render_widget(boards_para, inner_layout[3]);

    let clocks_hdr = Paragraph::new(Line::from(Span::styled(
        "Clocks",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )));
    frame.render_widget(clocks_hdr, inner_layout[4]);

    let clocks_body: Vec<Line> = if project.clock_ids.is_empty() {
        vec![Line::from(Span::styled(
            "  (none)",
            Style::default().fg(Color::DarkGray),
        ))]
    } else {
        project
            .clock_ids
            .iter()
            .map(|id| {
                let matched = app.config.clocks.iter().find(|c| &c.id == id);
                match matched {
                    Some(c) => Line::from(format!("  {}  {}  ({})", c.id, c.email, c.base_url)),
                    None => Line::from(format!("  {}  (missing)", id)),
                }
            })
            .collect()
    };
    let clocks_para = Paragraph::new(clocks_body).wrap(Wrap { trim: false });
    frame.render_widget(clocks_para, inner_layout[5]);

    let pending_count = app
        .pending
        .entries
        .iter()
        .filter(|e| e.project_code == project.code)
        .count();
    let pending_line = Paragraph::new(Line::from(vec![
        Span::styled("Pending entries: ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            pending_count.to_string(),
            Style::default().fg(Color::Yellow),
        ),
    ]));
    frame.render_widget(pending_line, inner_layout[7]);
}

pub fn keybindings_hint() -> Vec<(&'static str, &'static str)> {
    vec![
        ("j/k", "nav"),
        ("a", "add"),
        ("d", "unlink"),
        ("Tab", "next tab"),
        ("q", "quit"),
    ]
}
