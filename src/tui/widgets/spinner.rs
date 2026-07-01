const FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

#[derive(Debug, Default)]
pub struct SpinnerState {
    frame: usize,
}

impl SpinnerState {
    pub fn advance(&mut self) {
        self.frame = (self.frame + 1) % FRAMES.len();
    }

    pub fn glyph(&self) -> &'static str {
        FRAMES[self.frame % FRAMES.len()]
    }
}
