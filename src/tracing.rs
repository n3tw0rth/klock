use std::{
    env,
    fs::{self},
};

use tracing_subscriber::{filter, layer::SubscriberExt, util::SubscriberInitExt, Layer};

use crate::error::Error;

use super::error::Result;

pub struct Tracer {}
impl Tracer {
    pub fn init() -> Result<()> {
        let file_path = dirs::data_dir()
            .map(|mut path| {
                path.push(env!("CARGO_PKG_NAME")); // append the package name
                path.push("runtime.log");
                path
            })
            .unwrap();

        if let Some(parent) = file_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }

        let file_layer = tracing_subscriber::fmt::layer().compact().with_writer(
            fs::OpenOptions::new()
                .append(true)
                .create(true)
                .open(&file_path)?,
        );

        tracing_subscriber::registry()
            .with(
                file_layer
                    .with_ansi(true)
                    .with_target(true)
                    .with_filter(filter::LevelFilter::INFO),
            )
            .try_init()
            .map_err(|e| Error::CustomError(e.to_string()))?;
        Ok(())
    }
}
