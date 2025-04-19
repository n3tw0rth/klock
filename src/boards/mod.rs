use async_trait::async_trait;
pub mod clickup;
pub mod jira;

use crate::Args;
use crate::error::Result;
#[async_trait]
pub trait Board {
    async fn new() -> Self;
    async fn init(self, args: Args) -> Result<()>;
    async fn get_project_issues(&self, project_code: &String) -> Result<()>;
    async fn process_arguments(&mut self, args: Args) -> Result<()>;
    async fn logout(&self) -> Result<()>;
    async fn fuzzy_search(&mut self, project_code: &String, pattern: &String) -> Result<()>;
}
