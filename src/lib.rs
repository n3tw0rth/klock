pub mod app;
pub mod boards;
pub mod clocks;
pub mod common;
pub mod error;
pub mod prelude;

use clap::Parser;

/// Arguments for the cli
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Logout, removes user credential from local machine
    /// user must login again to use the cli
    #[arg(short, long)]
    logout: Option<bool>,
}
