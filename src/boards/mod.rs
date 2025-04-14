use async_trait::async_trait;
pub mod clickup;
pub mod jira;

use crate::Args;
use crate::error::Result;
#[async_trait]
pub trait Board {
    async fn new() -> Self;
    async fn init(self, args: Args) -> Result<()>;
    async fn get_project_issues(&self, project_code: &str) -> Result<()>;
    async fn process_arguments(&self, args: Args) -> Result<()>;
    async fn logout(&self) -> Result<()>;
}
