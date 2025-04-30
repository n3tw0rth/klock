use dirs::config_dir;
use serde::Deserialize;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
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

#[derive(Default, Deserialize, Debug)]
pub struct AppConfig {
    clocks: Vec<String>,
    editor: Option<String>,
    // TODO: Temporarily commented out – planned for future use.
    // project: Option<ProjectCodes>,
}

#[derive(Default, Deserialize, Debug)]
struct ProjectCodes {
    // TODO: Temporarily commented out – planned for future use.
    // codes: HashMap<String, String>,
}
