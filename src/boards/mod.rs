pub mod clickup;
pub mod jira;

use crate::error::Result;
trait Board {
    async fn new() -> Self;
    async fn init(self) -> Result<()>;
    async fn issues(&self) -> Result<()>;
}
