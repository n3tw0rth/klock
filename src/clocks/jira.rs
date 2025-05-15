use async_trait::async_trait;

use super::Clock;
use crate::{error::Result, Args};

struct JiraClock {}

#[async_trait]
/// https://developer.atlassian.com/cloud/jira/platform/rest/v3/api-group-issue-worklogs/#api-group-issue-worklogs
impl Clock for JiraClock {
    async fn new() -> Self {
        Self {}
    }

    async fn init(&mut self, _args: Args) -> Result<()> {
        Ok(())
    }

    async fn process_arguments(&mut self, _args: Args) -> Result<()> {
        Ok(())
    }

    async fn log(&self) -> Result<()> {
        Ok(())
    }
}
