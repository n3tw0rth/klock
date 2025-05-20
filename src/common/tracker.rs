use std::path::PathBuf;

use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use shell_words;
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
        title: String,
    ) -> Result<()> {
        println!("logging time for {}", task);
        info!("create new entry");

        // validate if the end time is after and start
        if start > end && !end.eq("-1") {
            return Err(Error::CustomError(
                "End time must be later than start time.".to_string(),
            ));
        }

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

        let new_entry = format!(
            "{} {} {} {} \"{}\"\n",
            project_code, task, end, start, title
        );

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

        let tokens: Vec<String> =
            shell_words::split(&line).map_err(|e| Error::CustomError(e.to_string()))?;

        // skip if there are no current tasks
        if tokens.len() != 5 {
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
            tokens.get(4).unwrap().to_string(),
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

    /// Read the day's log file to by the clocks to log the time on respective platform
    pub async fn read(&self) -> Result<Vec<String>> {
        let file = fs::File::open(&self.file).await?;
        let reader = BufReader::new(file);

        let mut lines = vec![];
        let mut line_stream = tokio::io::BufReader::new(reader).lines();

        while let Some(line) = line_stream.next_line().await? {
            lines.push(line);
        }

        Ok(lines)
    }

    ///// This method is used to forcefully stop a ongoing tasks assuming users will not work on
    ///// multiple projects at once
    //pub async fn force_terminate_tasks(&self) -> Result<()> {
    //    self.stop_current(String::from("-1")).await?;
    //    Ok(())
    //}

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

    /// Convert string in 24hrs system to yyyy-MM-ddThh:mm:ssZ
    pub fn format_24_hrs(&self, time_str: &str) -> Result<String> {
        let timezone_hours: f32 = self.config.time_zone.unwrap_or(0.0);
        let date = NaiveDate::parse_from_str(
            &std::env::var("JIRED_CURRENT_TIME")
                .unwrap_or(chrono::Local::now().format("%Y-%m-%d").to_string()),
            "%Y-%m-%d",
        )
        .map_err(|_| Error::CustomError("Error parsing time".to_string()))?;

        let hour: u32 = time_str[0..2]
            .parse()
            .map_err(|_| Error::CustomError("Error parsing time".to_string()))?;
        let minute: u32 = time_str[2..4]
            .parse()
            .map_err(|_| Error::CustomError("Error parsing time".to_string()))?;

        let time = NaiveTime::from_hms_opt(hour, minute, 0)
            .ok_or("invalid time components")
            .map_err(|e| Error::CustomError(e.to_string()))?;

        let naive_datetime = date.and_time(time);

        let hours = timezone_hours.trunc() as i32;
        let minutes = ((timezone_hours - hours as f32) * 60.0) as i32;
        let offset_seconds = hours * 3600 + minutes * 60;

        // Interpret the naive datetime as being in the specified timezone
        // First create a UTC datetime by subtracting the offset
        let adjusted_naive_datetime =
            naive_datetime - chrono::Duration::seconds(offset_seconds as i64);

        // Convert to UTC datetime
        #[allow(deprecated)]
        let utc_datetime = DateTime::<Utc>::from_utc(adjusted_naive_datetime, Utc);

        // Return in RFC3339 format
        Ok(utc_datetime.to_rfc3339())
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
                "Task title".to_string(),
            )
            .await
            .unwrap();

        let mut contents = String::new();
        let mut f = fs::File::open(&file_path).await.unwrap();
        f.read_to_string(&mut contents).await.unwrap();

        assert_eq!(contents.trim(), "proj PROJ-3 1234 1130 \"Task title\"");
    }
}
