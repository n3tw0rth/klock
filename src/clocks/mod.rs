pub mod clickup;
pub mod clockify;
pub mod jira;

use async_trait::async_trait;

use crate::error::Result;

#[async_trait]
pub trait Clock {
    async fn new() -> Self;
    async fn init(&mut self) -> Result<()>;
    async fn log() -> Result<()>;
}
