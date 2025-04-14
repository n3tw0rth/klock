use super::Board;

use crate::error::Result;

struct ClickUp {}
impl Board for ClickUp {
    async fn new() -> Self {
        unimplemented!()
    }
    async fn init(self) -> Result<()> {
        unimplemented!()
    }
    async fn issues(&self) -> Result<()> {
        unimplemented!()
    }
}
