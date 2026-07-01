use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui::Frame;

use crate::boards::{RemoteProject, RemoteTask};
use crate::config::models::{BoardPlatform, ClockPlatform, ProjectConfig};
use crate::services::projects::ClockOption;
use crate::tui::app::App;
use crate::tui::modal::{
    AuthDraft, AuthFieldFocus, EditPendingFocus, Modal, PendingEditDraft,
};
use crate::tui::widgets::date_picker::DatePickerState;
use crate::tui::widgets::time_input::TimeInputState;

pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

pub fn draw(frame: &mut Frame, app: &mut App, area: Rect) {
    let Some(modal) = app.modals.last() else { return; };
    match modal {
        Modal::Confirm { title, body, .. } => {
            super::confirm::draw(frame, area, title, body);
        }
        Modal::EditPending {
            idx, fields, focus, ..
        } => {
            draw_edit_pending(frame, area, *idx, fields, *focus);
        }
        Modal::AddProjectCode { input } => {
            draw_text_prompt(
                frame,
                area,
                "Add project — enter project code",
                "Local code used in klock (e.g. ALPHA). Enter to continue, Esc to cancel.",
                input,
                false,
            );
        }
        Modal::AddProjectKind { code, state } => {
            let items: Vec<String> = vec!["Board".into(), "Clock".into()];
            let mut list_state = state.clone();
            draw_pick_list(
                frame,
                area,
                &format!("Add integration to {code}"),
                "Board attaches remote tasks; Clock attaches a time destination.",
                &items,
                &mut list_state,
            );
        }
        Modal::AddProjectPickBoard { code, options, state } => {
            let items: Vec<String> = options
                .iter()
                .map(|b| format!("{}  {}  ({})", b.id, b.email, b.base_url))
                .collect();
            let mut list_state = state.clone();
            draw_pick_list(
                frame,
                area,
                &format!("Pick board for {code}"),
                "Enter to search projects on this board, Esc to cancel.",
                &items,
                &mut list_state,
            );
        }
        Modal::AddProjectPickClock { code, options, state } => {
            let items: Vec<String> = options.iter().map(clock_option_label).collect();
            let mut list_state = state.clone();
            draw_pick_list(
                frame,
                area,
                &format!("Pick clock for {code}"),
                "Enter to link, Esc to cancel.",
                &items,
                &mut list_state,
            );
        }
        Modal::AddProjectSearchQuery { code, input, .. } => {
            draw_text_prompt(
                frame,
                area,
                &format!("Search projects for {code}"),
                "Enter a search query; leave blank for all. Enter to search, Esc to cancel.",
                input,
                false,
            );
        }
        Modal::AddProjectSearchResults {
            code, results, state, loading, ..
        } => {
            draw_search_results(
                frame,
                area,
                &format!("Select project → {code}"),
                results,
                state.clone(),
                *loading,
                app,
            );
        }
        Modal::PickIntegrationToRemove { code, options, state } => {
            let items: Vec<String> = options
                .iter()
                .map(|r| format!("[{}] {}", r.kind.label(), r.label))
                .collect();
            let mut list_state = state.clone();
            draw_pick_list(
                frame,
                area,
                &format!("Unlink integration from {code}"),
                "Enter to unlink (with confirm), Esc to cancel.",
                &items,
                &mut list_state,
            );
        }
        Modal::PickProject { state } => {
            let items: Vec<String> = app
                .config
                .projects
                .iter()
                .map(project_label)
                .collect();
            let mut list_state = state.clone();
            draw_pick_list(
                frame,
                area,
                "Pick project",
                "Enter to search tasks, Esc to cancel.",
                &items,
                &mut list_state,
            );
        }
        Modal::SearchTaskQuery { project_code, input, .. } => {
            draw_text_prompt(
                frame,
                area,
                &format!("Search tasks in {project_code}"),
                "Enter a search query. Enter to search, Esc to cancel.",
                input,
                false,
            );
        }
        Modal::SearchTaskResults {
            project_code,
            results,
            filtered,
            query,
            state,
            loading,
            ..
        } => {
            draw_task_results(
                frame,
                area,
                project_code,
                results,
                filtered,
                query,
                state.clone(),
                *loading,
                app,
            );
        }
        Modal::TimeStart { draft, input } => {
            draw_time_prompt(
                frame,
                area,
                &format!("Start time — [{}] {}", draft.task.key, draft.task.title),
                "HHMM (e.g. 0930). Blank or 'now' for the current time. Enter to continue.",
                input,
            );
        }
        Modal::TimeEnd { draft, input } => {
            draw_time_prompt(
                frame,
                area,
                &format!("End time (optional) — [{}] {}", draft.task.key, draft.task.title),
                "HHMM or blank to leave open. Enter to start session.",
                input,
            );
        }
        Modal::StopTime { input } => {
            draw_time_prompt(
                frame,
                area,
                "Stop session at",
                "HHMM or 'now'. Enter to queue a pending entry.",
                input,
            );
        }
        Modal::AuthBoardPlatform { state } => {
            let items: Vec<String> = vec!["Jira".into(), "ClickUp".into()];
            let mut list_state = state.clone();
            draw_pick_list(
                frame,
                area,
                "Add board integration — pick platform",
                "Enter to continue, Esc to cancel.",
                &items,
                &mut list_state,
            );
        }
        Modal::AuthClockPlatform { state } => {
            let items: Vec<String> = vec!["Jira".into(), "Clockify".into()];
            let mut list_state = state.clone();
            draw_pick_list(
                frame,
                area,
                "Add clock integration — pick platform",
                "Enter to continue, Esc to cancel.",
                &items,
                &mut list_state,
            );
        }
        Modal::AuthLoginForm { draft, focus, error } => {
            draw_auth_form(frame, area, draft, *focus, error.as_deref());
        }
        Modal::DatePicker { state } => {
            draw_date_picker(frame, area, state);
        }
    }
}

fn draw_date_picker(frame: &mut Frame, area: Rect, state: &DatePickerState) {
    let rect = centered_rect(50, 30, area);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Active date ")
        .border_style(Style::default().fg(Color::Cyan));
    frame.render_widget(Clear, rect);
    frame.render_widget(block.clone(), rect);
    let inner = block.inner(rect);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(inner);

    let hint = Paragraph::new("YYYY-MM-DD. Enter to save, Esc to cancel.")
        .style(Style::default().fg(Color::DarkGray))
        .wrap(Wrap { trim: true });
    frame.render_widget(hint, layout[0]);

    super::text_input::draw(frame, layout[1], &state.input, "Date", true, false);

    let err = Paragraph::new(state.error.clone().unwrap_or_default())
        .style(Style::default().fg(Color::Red))
        .alignment(Alignment::Center);
    frame.render_widget(err, layout[2]);

    let footer = Paragraph::new(Line::from(vec![
        Span::styled(" Enter ", key_style()),
        Span::raw("  save    "),
        Span::styled(" Esc ", key_style()),
        Span::raw("  cancel"),
    ]))
    .alignment(Alignment::Center);
    frame.render_widget(footer, layout[3]);
}

fn draw_auth_form(
    frame: &mut Frame,
    area: Rect,
    draft: &AuthDraft,
    focus: AuthFieldFocus,
    error: Option<&str>,
) {
    let title = match draft {
        AuthDraft::Board { platform, .. } => match platform {
            BoardPlatform::Jira => "Login — Jira board",
            BoardPlatform::ClickUp => "Login — ClickUp board",
        },
        AuthDraft::Clock { platform, .. } => match platform {
            ClockPlatform::Jira => "Login — Jira clock",
            ClockPlatform::Clockify => "Login — Clockify clock",
        },
    };
    let visible = draft.visible_fields();
    let rect = centered_rect(60, 70, area);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {title} "))
        .border_style(Style::default().fg(Color::Cyan));
    frame.render_widget(Clear, rect);
    frame.render_widget(block.clone(), rect);
    let inner = block.inner(rect);

    let mut constraints: Vec<Constraint> = visible.iter().map(|_| Constraint::Length(3)).collect();
    constraints.push(Constraint::Length(1));
    constraints.push(Constraint::Length(1));
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(inner);

    for (i, field) in visible.iter().enumerate() {
        let is_focused = *field == focus;
        let (input, masked) = field_input(draft, *field);
        super::text_input::draw(frame, layout[i], input, field.label(), is_focused, masked);
    }

    let err_row = layout.len() - 2;
    let err = Paragraph::new(error.unwrap_or_default().to_string())
        .style(Style::default().fg(Color::Red))
        .alignment(Alignment::Center);
    frame.render_widget(err, layout[err_row]);

    let footer_row = layout.len() - 1;
    let footer = Paragraph::new(Line::from(vec![
        Span::styled(" Tab ", key_style()),
        Span::raw("  next field    "),
        Span::styled(" Enter ", key_style()),
        Span::raw("  save    "),
        Span::styled(" Esc ", key_style()),
        Span::raw("  cancel"),
    ]))
    .alignment(Alignment::Center);
    frame.render_widget(footer, layout[footer_row]);
}

fn field_input<'a>(draft: &'a AuthDraft, field: AuthFieldFocus) -> (&'a tui_input::Input, bool) {
    match draft {
        AuthDraft::Board {
            base_url,
            email,
            team_id,
            secret,
            ..
        } => match field {
            AuthFieldFocus::BaseUrl => (base_url, false),
            AuthFieldFocus::Email => (email, false),
            AuthFieldFocus::TeamId => (team_id, false),
            AuthFieldFocus::Secret => (secret, true),
        },
        AuthDraft::Clock {
            base_url,
            email,
            secret,
            ..
        } => match field {
            AuthFieldFocus::BaseUrl => (base_url, false),
            AuthFieldFocus::Email => (email, false),
            AuthFieldFocus::Secret => (secret, true),
            AuthFieldFocus::TeamId => (base_url, false),
        },
    }
}

fn project_label(p: &ProjectConfig) -> String {
    if p.platform_project_name.is_empty() {
        p.code.clone()
    } else {
        format!("{}  —  {}", p.code, p.platform_project_name)
    }
}

fn clock_option_label(opt: &ClockOption) -> String {
    match opt {
        ClockOption::Existing(id) => id.clone(),
        ClockOption::Derive { board_id } => {
            format!("+ Jira worklog (from board: {board_id})")
        }
    }
}

fn draw_text_prompt(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    hint: &str,
    input: &tui_input::Input,
    masked: bool,
) {
    let rect = centered_rect(60, 30, area);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {title} "))
        .border_style(Style::default().fg(Color::Cyan));
    frame.render_widget(Clear, rect);
    frame.render_widget(block.clone(), rect);
    let inner = block.inner(rect);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(inner);

    let hint_para = Paragraph::new(hint.to_string())
        .style(Style::default().fg(Color::DarkGray))
        .wrap(Wrap { trim: true });
    frame.render_widget(hint_para, layout[0]);

    super::text_input::draw(frame, layout[1], input, "Input", true, masked);

    let footer = Paragraph::new(Line::from(vec![
        Span::styled(" Enter ", key_style()),
        Span::raw("  continue    "),
        Span::styled(" Esc ", key_style()),
        Span::raw("  cancel"),
    ]))
    .alignment(Alignment::Center);
    frame.render_widget(footer, layout[3]);
}

fn draw_pick_list(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    hint: &str,
    items: &[String],
    state: &mut ratatui::widgets::ListState,
) {
    let rect = centered_rect(60, 60, area);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {title} "))
        .border_style(Style::default().fg(Color::Cyan));
    frame.render_widget(Clear, rect);
    frame.render_widget(block.clone(), rect);
    let inner = block.inner(rect);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(3),
            Constraint::Length(1),
        ])
        .split(inner);

    let hint_para = Paragraph::new(hint.to_string())
        .style(Style::default().fg(Color::DarkGray))
        .wrap(Wrap { trim: true });
    frame.render_widget(hint_para, layout[0]);

    if items.is_empty() {
        let empty = Paragraph::new("Nothing to choose from.")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);
        frame.render_widget(empty, layout[1]);
    } else {
        super::single_select::draw(frame, layout[1], items, state, "", true);
    }

    let footer = Paragraph::new(Line::from(vec![
        Span::styled(" j/k ", key_style()),
        Span::raw("  nav    "),
        Span::styled(" Enter ", key_style()),
        Span::raw("  select    "),
        Span::styled(" Esc ", key_style()),
        Span::raw("  cancel"),
    ]))
    .alignment(Alignment::Center);
    frame.render_widget(footer, layout[2]);
}

fn draw_time_prompt(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    hint: &str,
    state: &TimeInputState,
) {
    let rect = centered_rect(60, 30, area);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {title} "))
        .border_style(Style::default().fg(Color::Cyan));
    frame.render_widget(Clear, rect);
    frame.render_widget(block.clone(), rect);
    let inner = block.inner(rect);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(inner);

    let hint_para = Paragraph::new(hint.to_string())
        .style(Style::default().fg(Color::DarkGray))
        .wrap(Wrap { trim: true });
    frame.render_widget(hint_para, layout[0]);

    super::text_input::draw(frame, layout[1], &state.input, "HHMM", true, false);

    let err = Paragraph::new(state.error.clone().unwrap_or_default())
        .style(Style::default().fg(Color::Red))
        .alignment(Alignment::Center);
    frame.render_widget(err, layout[2]);

    let footer = Paragraph::new(Line::from(vec![
        Span::styled(" Enter ", key_style()),
        Span::raw("  continue    "),
        Span::styled(" Esc ", key_style()),
        Span::raw("  cancel"),
    ]))
    .alignment(Alignment::Center);
    frame.render_widget(footer, layout[3]);
}

fn draw_task_results(
    frame: &mut Frame,
    area: Rect,
    project_code: &str,
    results: &[RemoteTask],
    filtered: &[usize],
    query: &tui_input::Input,
    state: ratatui::widgets::ListState,
    loading: bool,
    app: &App,
) {
    let rect = centered_rect(75, 70, area);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" Pick task in {project_code} "))
        .border_style(Style::default().fg(Color::Cyan));
    frame.render_widget(Clear, rect);
    frame.render_widget(block.clone(), rect);
    let inner = block.inner(rect);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(1),
            Constraint::Min(3),
            Constraint::Length(1),
        ])
        .split(inner);

    super::text_input::draw(frame, layout[0], query, "Filter", true, false);

    let status = if loading {
        Paragraph::new(Line::from(vec![
            Span::styled(app.spinner.glyph().to_string(), Style::default().fg(Color::Yellow)),
            Span::raw("  Searching…"),
        ]))
    } else {
        Paragraph::new(format!(
            "{}/{} match(es).",
            filtered.len(),
            results.len()
        ))
        .style(Style::default().fg(Color::DarkGray))
    };
    frame.render_widget(status, layout[1]);

    if filtered.is_empty() && !loading {
        let empty = Paragraph::new("No tasks match.")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);
        frame.render_widget(empty, layout[2]);
    } else {
        let items: Vec<String> = filtered
            .iter()
            .map(|i| {
                let t = &results[*i];
                format!("[{}] {}  ({})", t.key, t.title, t.status)
            })
            .collect();
        let mut ls = state;
        super::single_select::draw(frame, layout[2], &items, &mut ls, "", true);
    }

    let footer = Paragraph::new(Line::from(vec![
        Span::styled(" ↑↓ ", key_style()),
        Span::raw("  nav    "),
        Span::styled(" Enter ", key_style()),
        Span::raw("  select    "),
        Span::styled(" Esc ", key_style()),
        Span::raw("  cancel"),
    ]))
    .alignment(Alignment::Center);
    frame.render_widget(footer, layout[3]);
}

fn draw_search_results(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    results: &[RemoteProject],
    state: ratatui::widgets::ListState,
    loading: bool,
    app: &App,
) {
    let rect = centered_rect(70, 60, area);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {title} "))
        .border_style(Style::default().fg(Color::Cyan));
    frame.render_widget(Clear, rect);
    frame.render_widget(block.clone(), rect);
    let inner = block.inner(rect);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(3),
            Constraint::Length(1),
        ])
        .split(inner);

    let hint_para = if loading {
        Paragraph::new(Line::from(vec![
            Span::styled(app.spinner.glyph().to_string(), Style::default().fg(Color::Yellow)),
            Span::raw("  Searching…"),
        ]))
    } else {
        Paragraph::new(format!("{} result(s).", results.len()))
            .style(Style::default().fg(Color::DarkGray))
    };
    frame.render_widget(hint_para, layout[0]);

    if results.is_empty() && !loading {
        let empty = Paragraph::new("No projects found.")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);
        frame.render_widget(empty, layout[1]);
    } else {
        let items: Vec<String> = results
            .iter()
            .map(|p| format!("[{}] {}", p.key, p.name))
            .collect();
        let mut ls = state;
        super::single_select::draw(frame, layout[1], &items, &mut ls, "", true);
    }

    let footer = Paragraph::new(Line::from(vec![
        Span::styled(" j/k ", key_style()),
        Span::raw("  nav    "),
        Span::styled(" Enter ", key_style()),
        Span::raw("  link    "),
        Span::styled(" Esc ", key_style()),
        Span::raw("  cancel"),
    ]))
    .alignment(Alignment::Center);
    frame.render_widget(footer, layout[2]);
}

fn draw_edit_pending(
    frame: &mut Frame,
    area: Rect,
    idx: u32,
    fields: &PendingEditDraft,
    focus: EditPendingFocus,
) {
    let rect = centered_rect(60, 60, area);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" Edit pending #{idx} "))
        .border_style(Style::default().fg(Color::Cyan));
    frame.render_widget(Clear, rect);
    frame.render_widget(block.clone(), rect);
    let inner = block.inner(rect);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(inner);

    super::text_input::draw(
        frame,
        layout[0],
        &fields.hours,
        EditPendingFocus::Hours.label(),
        focus == EditPendingFocus::Hours,
        false,
    );
    super::text_input::draw(
        frame,
        layout[1],
        &fields.start,
        EditPendingFocus::Start.label(),
        focus == EditPendingFocus::Start,
        false,
    );
    super::text_input::draw(
        frame,
        layout[2],
        &fields.end,
        EditPendingFocus::End.label(),
        focus == EditPendingFocus::End,
        false,
    );
    super::text_input::draw(
        frame,
        layout[3],
        &fields.description,
        EditPendingFocus::Description.label(),
        focus == EditPendingFocus::Description,
        false,
    );

    let err = Paragraph::new(fields.error.clone().unwrap_or_default())
        .style(Style::default().fg(Color::Red))
        .alignment(Alignment::Center);
    frame.render_widget(err, layout[4]);

    let footer = Paragraph::new(Line::from(vec![
        Span::styled(" Tab ", key_style()),
        Span::raw("  next field    "),
        Span::styled(" Enter ", key_style()),
        Span::raw("  save    "),
        Span::styled(" Esc ", key_style()),
        Span::raw("  cancel"),
    ]))
    .alignment(Alignment::Center);
    frame.render_widget(footer, layout[5]);
}

fn key_style() -> Style {
    Style::default()
        .bg(Color::DarkGray)
        .fg(Color::White)
        .add_modifier(Modifier::BOLD)
}
