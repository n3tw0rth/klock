use super::Clock;
use crate::error::Result;

struct JiraClock {}

impl Clock for JiraClock {
    async fn new() -> Self {
        unimplemented!()
    }
    async fn log() -> Result<()> {
        unimplemented!()
    }
}
