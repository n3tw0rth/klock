use crossterm::event::{Event as CtEvent, KeyEvent};
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;
use tui_input::backend::crossterm::EventHandler;
use tui_input::Input;

pub fn handle_key(input: &mut Input, key: KeyEvent) -> bool {
    input.handle_event(&CtEvent::Key(key)).is_some()
}

pub fn draw(
    frame: &mut Frame,
    area: Rect,
    input: &Input,
    title: &str,
    focused: bool,
    masked: bool,
) {
    let border_style = if focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::Gray)
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {title} "))
        .border_style(border_style);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let width = inner.width.max(1) as usize;
    let scroll = input.visual_scroll(width);
    let display: String = if masked {
        "•".repeat(input.value().chars().count())
    } else {
        input.value().to_string()
    };
    let para = Paragraph::new(display).scroll((0, scroll as u16));
    frame.render_widget(para, inner);

    if focused {
        let cursor_col = input.visual_cursor().saturating_sub(scroll);
        let x = inner.x + cursor_col as u16;
        frame.set_cursor_position((x, inner.y));
    }
}
