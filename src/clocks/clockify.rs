use super::Clock;
use crate::error::Result;

struct ClockifyClock {}

impl Clock for ClockifyClock {
    async fn new() -> Self {
        unimplemented!()
    }
    async fn log() -> Result<()> {
        unimplemented!()
    }
}
