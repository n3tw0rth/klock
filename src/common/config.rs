use dirs::config_dir;
use serde::Deserialize;
use std::collections::HashMap;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use toml;

use crate::error::{Error::CustomError, Result};

pub struct ConfigParser {
    config: AppConfig,
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

        Ok(Self {
            config: AppConfig::default(),
        })
    }
}

#[derive(Default, Deserialize, Debug)]
struct AppConfig {
    clocks: Vec<String>,
    project: Option<ProjectCodes>,
}

#[derive(Default, Deserialize, Debug)]
struct ProjectCodes {
    codes: HashMap<String, String>,
}
