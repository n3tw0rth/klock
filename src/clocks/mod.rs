pub mod clickup;
pub mod clockify;
pub mod jira;

use async_trait::async_trait;

use crate::error::Result;
use crate::Args;

#[async_trait]
pub trait Clock {
    async fn new() -> Self;
    async fn init(&mut self, args: Args) -> Result<()>;
    async fn process_arguments(&mut self, args: Args) -> Result<()>;
    async fn log(&self) -> Result<()>;
    async fn logout(&self) -> Result<()>;
}
