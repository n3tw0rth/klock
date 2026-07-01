use std::time::{Duration, Instant};

use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;
use unicode_width::UnicodeWidthStr;

#[derive(Debug, Clone, Copy)]
pub enum ToastKind {
    Info,
    Success,
    Warn,
    Error,
}

impl ToastKind {
    fn color(self) -> Color {
        match self {
            ToastKind::Info => Color::Cyan,
            ToastKind::Success => Color::Green,
            ToastKind::Warn => Color::Yellow,
            ToastKind::Error => Color::Red,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Toast {
    pub message: String,
    pub kind: ToastKind,
    pub expires_at: Instant,
}

impl Toast {
    pub fn new(message: impl Into<String>, kind: ToastKind, ttl: Duration) -> Self {
        Self {
            message: message.into(),
            kind,
            expires_at: Instant::now() + ttl,
        }
    }

    pub fn info(message: impl Into<String>) -> Self {
        Self::new(message, ToastKind::Info, Duration::from_secs(3))
    }

    pub fn success(message: impl Into<String>) -> Self {
        Self::new(message, ToastKind::Success, Duration::from_secs(3))
    }

    pub fn warn(message: impl Into<String>) -> Self {
        Self::new(message, ToastKind::Warn, Duration::from_secs(4))
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self::new(message, ToastKind::Error, Duration::from_secs(5))
    }

    pub fn is_expired(&self) -> bool {
        Instant::now() >= self.expires_at
    }
}

pub fn draw(frame: &mut Frame, toast: &Toast, area: Rect) {
    let width = (toast.message.width() as u16 + 4).min(area.width.saturating_sub(2));
    let height = 3u16;
    if area.width < width || area.height < height {
        return;
    }
    let x = area.x + area.width.saturating_sub(width).saturating_sub(1);
    let y = area.y + area.height.saturating_sub(height).saturating_sub(1);
    let rect = Rect {
        x,
        y,
        width,
        height,
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(toast.kind.color()));
    let para = Paragraph::new(Line::from(toast.message.clone()))
        .style(Style::default().add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(block);
    frame.render_widget(Clear, rect);
    frame.render_widget(para, rect);
}
