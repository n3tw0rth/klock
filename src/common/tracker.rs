use tokio::fs;
use tokio::io::AsyncWriteExt;

use crate::error::Error;
use crate::error::Result;

/// Tracker provides the time tracking layer for the program, Store records on the local filesystem
/// and different layers can access the time logs thru tracker
#[derive(Default, Debug)]
pub struct Tracker {
    /// time logs for each day will be saved on a seperate file
    current_file: String,
}

impl Tracker {
    pub async fn new() -> Self {
        let mut filename: String = chrono::Local::now()
            .to_string()
            .split_once(" ")
            .expect("Failed to get the local date")
            .0
            .to_string();

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

        Self {
            current_file: file_path
                .and_then(|p| p.to_str().map(|s| s.to_string()))
                .expect("Failed to find the data directory"),
        }
    }

    /// Creates a new entry on the log file
    pub async fn create_entry(
        &self,
        project_code: String,
        task: String,
        end: String,
        start: String,
    ) -> Result<()> {
        println!("{}", self.current_file);
        let mut file = fs::OpenOptions::new()
            .append(true)
            .read(true)
            .open(&self.current_file)
            .await?;

        let new_entry = format!("{} {} {} {}\n", project_code, task, end, start);

        file.write_all(new_entry.as_bytes()).await?;
        file.flush().await?;

        Ok(())
    }

    /// Manages the current ongoing tasks state till it stops
    /// the state will be mantained in a different file named current.jj
    pub async fn handle_current_task(&self) {
        unimplemented!()
    }

    /// Let the user to open up a log file to edit manually
    /// will open the file for the day if the date is not set
    pub async fn open_editor(&self) {
        unimplemented!()
    }
}
