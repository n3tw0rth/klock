use chrono::NaiveDate;
use crossterm::event::KeyEvent;
use tui_input::Input;

#[derive(Debug, Clone)]
pub struct DatePickerState {
    pub input: Input,
    pub error: Option<String>,
}

impl DatePickerState {
    pub fn new(seed: NaiveDate) -> Self {
        Self {
            input: Input::new(seed.format("%Y-%m-%d").to_string()),
            error: None,
        }
    }

    pub fn resolve(&self) -> Result<NaiveDate, String> {
        NaiveDate::parse_from_str(self.input.value().trim(), "%Y-%m-%d")
            .map_err(|_| "Use YYYY-MM-DD (e.g. 2026-07-01)".to_string())
    }
}

pub fn handle_key(state: &mut DatePickerState, key: KeyEvent) -> bool {
    let handled = super::text_input::handle_key(&mut state.input, key);
    if handled {
        state.error = None;
    }
    handled
}
