use std::time::Duration;

use crossterm::event::{Event as CtEvent, EventStream};
use futures::StreamExt;
use tokio::sync::mpsc;
use tokio::time::{interval, Interval};

use crate::tui::messages::ServiceResult;

pub enum Event {
    Key(crossterm::event::KeyEvent),
    Resize(u16, u16),
    Tick,
    Service(ServiceResult),
}

pub struct EventBus {
    events: EventStream,
    tick: Interval,
    results: mpsc::Receiver<ServiceResult>,
}

impl EventBus {
    pub fn new(results: mpsc::Receiver<ServiceResult>) -> Self {
        let mut tick = interval(Duration::from_millis(1000));
        tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        Self {
            events: EventStream::new(),
            tick,
            results,
        }
    }

    pub async fn next(&mut self) -> Option<Event> {
        tokio::select! {
            ev = self.events.next() => match ev {
                Some(Ok(CtEvent::Key(k))) => Some(Event::Key(k)),
                Some(Ok(CtEvent::Resize(w, h))) => Some(Event::Resize(w, h)),
                Some(Ok(_)) => Some(Event::Tick),
                Some(Err(_)) => None,
                None => None,
            },
            _ = self.tick.tick() => Some(Event::Tick),
            msg = self.results.recv() => msg.map(Event::Service),
        }
    }
}
