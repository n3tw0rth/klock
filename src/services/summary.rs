use chrono::NaiveDate;

use crate::clocks::build_clock;
use crate::config::models::AppConfig;

#[derive(Debug, Clone)]
pub struct SummaryRow {
    pub project_code: String,
    pub project_name: String,
    pub clock_id: String,
    pub hours: Result<f32, String>,
}

pub async fn fetch_summary(cfg: &AppConfig, date: NaiveDate) -> Vec<SummaryRow> {
    let mut rows = Vec::new();
    for project in &cfg.projects {
        for clock_id in &project.clock_ids {
            let Some(clock_cfg) = cfg.clocks.iter().find(|c| &c.id == clock_id) else {
                rows.push(SummaryRow {
                    project_code: project.code.clone(),
                    project_name: project.platform_project_name.clone(),
                    clock_id: clock_id.clone(),
                    hours: Err(format!("clock '{clock_id}' missing from config")),
                });
                continue;
            };
            let clock = match build_clock(clock_cfg) {
                Ok(c) => c,
                Err(e) => {
                    rows.push(SummaryRow {
                        project_code: project.code.clone(),
                        project_name: project.platform_project_name.clone(),
                        clock_id: clock_id.clone(),
                        hours: Err(format!("init failed: {e}")),
                    });
                    continue;
                }
            };
            let hours = clock
                .get_logged_time(&project.platform_project_id, date)
                .await
                .map_err(|e| e.to_string());
            rows.push(SummaryRow {
                project_code: project.code.clone(),
                project_name: project.platform_project_name.clone(),
                clock_id: clock_id.clone(),
                hours,
            });
        }
    }
    rows
}
