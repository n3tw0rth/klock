use std::path::PathBuf;

use tokio::fs;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tracing::{info, instrument};

use crate::error::{Error, Result};

use super::config::{AppConfig, ConfigParser};

/// Tracker provides the time tracking layer for the program, Store records on the local filesystem
/// and different layers can access the time logs thru tracker
#[derive(Default, Debug)]
pub struct Tracker {
    /// time logs for each day will be saved on a seperate file
    file: String,
    config: AppConfig,
}

impl Tracker {
    pub async fn new() -> Result<Self> {
        let mut filename: String = std::env::var("JIRED_CURRENT_TIME")
            .unwrap_or(chrono::Local::now().format("%Y-%m-%d").to_string());

        filename.push_str(".jj");

        // Safely get data directory and build full path
        let file_path = dirs::data_dir().map(|mut path| {
            path.push(env!("CARGO_PKG_NAME")); // append the package name
            path.push(filename.clone()); // append the file name
            path
        });

        if let Some(path) = file_path.as_ref() {
            if !fs::try_exists(&path).await.unwrap_or(false) {
                // Ensure parent directories exist
                if let Some(parent) = path.parent() {
                    let _ = fs::create_dir_all(parent)
                        .await
                        .map_err(|e| Error::TrackerError(e.to_string()));
                }

                // Create the file (empty)
                let _ = fs::File::create(&path).await;
                println!("Created file at {}", path.display());
            }
        } else {
            Error::TrackerError("Cannot find the data directory in the host".to_string());
        };

        let config = ConfigParser::parse()
            .await
            .map_err(|e| Error::CustomError(e.to_string()))?
            .config;

        Ok(Self {
            file: file_path
                .and_then(|p| p.to_str().map(|s| s.to_string()))
                .expect("Failed to find the data directory"),
            config,
        })
    }

    /// Creates a new entry on the log file
    #[instrument]
    pub async fn create_entry(
        &self,
        project_code: &String,
        task: &String,
        end: String,
        start: String,
    ) -> Result<()> {
        println!("logging time for {}", task);
        let mut file = fs::OpenOptions::new()
            .append(true)
            .read(true)
            .open(&self.file)
            .await?;

        // if the end time of the task is set to default value, that task is ongoing, those tasks
        // should be written another file called current.jj
        if end == "-1" {
            file = fs::OpenOptions::new()
                .append(true)
                .read(true)
                .create(true)
                .open(self.get_current_file_path().await?)
                .await?;
        }

        let new_entry = format!("{} {} {} {}\n", project_code, task, end, start);

        file.write_all(new_entry.as_bytes()).await?;
        file.flush().await?;

        Ok(())
    }

    pub async fn get_current_file_path(&self) -> Result<PathBuf> {
        let mut current_file = PathBuf::from(&self.file)
            .parent()
            .expect("Cannot find the parent dir")
            .to_path_buf();

        current_file.push("current.jj");

        Ok(current_file)
    }

    /// Stops the current ongoing task, will be used mostly with the stop subcommand
    #[instrument(skip(self))]
    pub async fn stop_current(&self, at: String) -> Result<()> {
        info!("stopping the current task");
        let file = fs::OpenOptions::new()
            .append(true)
            .read(true)
            .create(true)
            .open(self.get_current_file_path().await?)
            .await?;
        let mut reader = BufReader::new(file);

        let mut line = String::new();
        reader.read_line(&mut line).await?;

        let tokens: Vec<String> = line.trim().split(" ").map(|v| v.to_string()).collect();
        let mut end_time = String::new();

        // To stop the current task immediately, when at value is not passed
        if at.eq("-1") {
            end_time = chrono::Local::now().format("%H%M").to_string();
        }
        // Stop the task on the value at
        else {
            end_time = at
        }

        self.create_entry(
            tokens.first().unwrap(),
            tokens.get(1).unwrap(),
            end_time,
            tokens.get(3).unwrap().to_string(),
        )
        .await?;

        // Drop the file handle before truncating
        drop(reader.into_inner());

        fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(self.get_current_file_path().await?)
            .await?;
        Ok(())
    }

    /// This method is used to forcefully stop a ongoing tasks assuming users will not work on
    /// multiple projects at once
    pub async fn force_terminate_tasks(&self) -> Result<()> {
        self.stop_current(String::from("-1")).await?;
        Ok(())
    }

    /// Let the user to open up a log file to edit manually
    /// will open the file for the day if the date is not set
    pub async fn open_editor(&self) {
        unimplemented!()
    }
}
