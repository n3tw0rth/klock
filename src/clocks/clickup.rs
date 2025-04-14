use super::Clock;
use crate::error::Result;
struct ClickUpClock {}

impl Clock for ClickUpClock {
    async fn new() -> Self {
        unimplemented!()
    }
    async fn log() -> Result<()> {
        unimplemented!()
    }
}
