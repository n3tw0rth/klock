mod app;
mod draw;
mod event;
mod keymap;
mod messages;
mod modal;
mod services;
mod state;
mod tabs;
mod terminal;
mod widgets;

use tokio::sync::mpsc;

use crate::config;
use crate::error::{KlockError, Result};

use self::app::App;
use self::draw::draw;
use self::event::{Event, EventBus};
use self::messages::{ServiceCommand, ServiceResult};
use self::terminal::{install_panic_hook, TerminalGuard};

pub async fn run() -> Result<()> {
    install_panic_hook();
    let mut guard = TerminalGuard::enter()?;

    let (cmd_tx, cmd_rx) = mpsc::channel::<ServiceCommand>(64);
    let (res_tx, res_rx) = mpsc::channel::<ServiceResult>(64);
    tokio::spawn(services::run(cmd_rx, res_tx));

    let cfg = config::load_config()?;
    let pending = config::load_pending()?;
    let session = config::load_session()?;
    let active_date = config::load_active_date()?;
    let mut app = App::new(cmd_tx, cfg, pending, session, active_date);
    app.kickoff_summary_fetch();

    let mut bus = EventBus::new(res_rx);
    while app.is_running() {
        guard
            .terminal
            .draw(|f| draw(f, &mut app))
            .map_err(|e| KlockError::ConfigError(format!("draw: {e}")))?;
        match bus.next().await {
            Some(Event::Key(k))
                if matches!(
                    k.code,
                    crossterm::event::KeyCode::Char('c')
                ) && k.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) =>
            {
                break;
            }
            Some(ev) => app.on_event(ev),
            None => break,
        }
    }
    Ok(())
}
