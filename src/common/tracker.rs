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

        let file_path = dirs::data_dir()
            .map(|mut path| {
                path.push(env!("CARGO_PKG_NAME"));
                path.push(filename.clone());
                path
            })
            .ok_or_else(|| Error::CustomError("Failed to get data dir".into()))?;

        let config = ConfigParser::parse()
            .await
            .map_err(|e| Error::CustomError(e.to_string()))?
            .config;

        Self::with_path_and_config(file_path, config).await
    }

    /// Creates a new entry on the log file
    pub async fn create_entry(
        &self,
        project_code: &String,
        task: &String,
        end: String,
        start: String,
    ) -> Result<()> {
        println!("logging time for {}", task);
        info!("create new entry");

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

        // skip if there are no current tasks
        if tokens.len() != 4 {
            return Ok(());
        }

        // To stop the current task immediately, when at value is not passed
        let end_time = if at.eq("-1") {
            chrono::Local::now().format("%H%M").to_string()
        }
        // Stop the task on the value at
        else {
            at
        };

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
        println!("Selected editor: {:?}", self.config.editor)
    }

    pub async fn with_path_and_config(file_path: PathBuf, config: AppConfig) -> Result<Self> {
        if !fs::try_exists(&file_path).await.unwrap_or(false) {
            if let Some(parent) = file_path.parent() {
                fs::create_dir_all(parent)
                    .await
                    .map_err(|e| Error::TrackerError(e.to_string()))?;
            }

            fs::File::create(&file_path).await?;
        }

        Ok(Self {
            file: file_path.to_string_lossy().to_string(),
            config,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use tokio::fs;
    use tokio::io::AsyncReadExt;

    fn dummy_config() -> AppConfig {
        AppConfig::default()
    }

    #[tokio::test]
    async fn test_create_entry_writes_to_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("log.jj");
        let tracker = Tracker::with_path_and_config(file_path.clone(), dummy_config())
            .await
            .unwrap();

        tracker
            .create_entry(
                &"proj".to_string(),
                &"PROJ-3".to_string(),
                "1234".to_string(),
                "1130".to_string(),
            )
            .await
            .unwrap();

        let mut contents = String::new();
        let mut f = fs::File::open(&file_path).await.unwrap();
        f.read_to_string(&mut contents).await.unwrap();

        assert_eq!(contents.trim(), "proj PROJ-3 1234 1130");
    }

    #[tokio::test]
    async fn test_force_terminate_tasks_creates_and_clears_current_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("log.jj");
        let tracker = Tracker::with_path_and_config(file_path.clone(), dummy_config())
            .await
            .unwrap();

        // Create an ongoing task
        tracker
            .create_entry(
                &"proj".to_string(),
                &"proj-3".to_string(),
                "-1".to_string(),
                "0900".to_string(),
            )
            .await
            .unwrap();

        // Make sure current.jj has the ongoing task
        let current_path = tracker.get_current_file_path().await.unwrap();
        let mut contents = String::new();
        let mut f = fs::File::open(&current_path).await.unwrap();
        f.read_to_string(&mut contents).await.unwrap();
        assert!(contents.contains("proj-3"));

        // Force terminate
        tracker.force_terminate_tasks().await.unwrap();

        // After termination, current.jj should be truncated
        let mut contents_after = String::new();
        let mut f = fs::File::open(&current_path).await.unwrap();
        f.read_to_string(&mut contents_after).await.unwrap();
        println!("{}", contents_after.trim());
        assert!(contents_after.trim().is_empty());
    }
}
