use dirs::config_dir;
use serde::Deserialize;
use tokio::fs::{self, File};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use toml;

use crate::error::{Error::CustomError, Result};

pub struct ConfigParser {
    pub config: AppConfig,
}

impl ConfigParser {
    pub async fn parse() -> Result<Self> {
        let mut config_file = config_dir().expect("Something wrong with config path");
        config_file.push(std::env!("CARGO_PKG_NAME"));
        config_file.push("config.toml");

        if !fs::try_exists(config_file.clone()).await? {
            let config_file_default_content = r#"clocks = ["jira","clockify"]
editor = "nvim"
            "#;
            // Create the file and write content
            let mut file = fs::File::create(&config_file).await?;
            file.write_all(config_file_default_content.as_bytes())
                .await?;
        }

        let mut content = String::new();
        let mut file = File::open(&config_file).await?;

        file.read_to_string(&mut content)
            .await
            .map_err(|e| CustomError(e.to_string()))?;

        let config: AppConfig = toml::from_str(&content).map_err(|e| CustomError(e.to_string()))?;

        Ok(Self { config })
    }

    pub fn get_clocks(self) -> Result<Vec<String>> {
        Ok(self.config.clocks)
    }

    pub fn get_editor(self) -> Result<String> {
        Ok(self.config.editor.unwrap_or("".to_string()))
    }
}

#[derive(Deserialize, Debug)]
pub struct AppConfig {
    pub clocks: Vec<String>,
    pub editor: Option<String>,
    // TODO: Temporarily commented out – planned for future use.
    // project: Option<ProjectCodes>,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            clocks: vec!["jira".to_string(), "clockify".to_string()],
            editor: None,
        }
    }
}

#[derive(Default, Deserialize, Debug)]
struct ProjectCodes {
    // TODO: Temporarily commented out – planned for future use.
    // codes: HashMap<String, String>,
}
