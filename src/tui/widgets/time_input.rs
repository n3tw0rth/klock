use crossterm::event::KeyEvent;
use tui_input::Input;

use crate::utils::time::parse_hhmm;

#[derive(Debug, Clone)]
pub struct TimeInputState {
    pub input: Input,
    pub error: Option<String>,
}

impl TimeInputState {
    pub fn new(initial: &str) -> Self {
        Self {
            input: Input::new(initial.to_string()),
            error: None,
        }
    }

    pub fn resolve(&self) -> Result<String, String> {
        parse_hhmm(self.input.value()).map_err(|e| e.to_string())
    }
}

pub fn handle_key(state: &mut TimeInputState, key: KeyEvent) -> bool {
    let handled = super::text_input::handle_key(&mut state.input, key);
    if handled {
        state.error = None;
    }
    handled
}
