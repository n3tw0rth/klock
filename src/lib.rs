pub mod app;
pub mod common;
pub mod error;
pub mod prelude;

use clap::Parser;

/// Arguments for the cli
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Resets the application state, removes the user provided secrets
    #[arg(long)]
    reset: Option<bool>,
}
