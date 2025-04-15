pub mod app;
pub mod boards;
pub mod clocks;
pub mod common;
pub mod error;
pub mod prelude;

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Define when a task starts
    Start {
        /// Project short code (eg: ticket id's prefix)
        #[arg(value_name = "projectcode")]
        project_name: String,

        /// Fuzzy text to search the right ticket, so you do not have remember the ticket id to log
        /// time
        #[arg(value_name = "pattern")]
        pattern: String,

        #[command(subcommand)]
        till: Option<StartSubcommandA>,
    },
    /// Define when to stop the current task
    Stop {
        /// Can use this argument to stop the current task at some point of the day instead of the
        /// current time
        #[arg(value_name = "at")]
        at: Option<String>,
    },
    /// Logout, removing the secrets
    Logout,
}

#[derive(Debug, Subcommand)]
pub enum StartSubcommandA {
    /// You can specify the time, will argument passed program will create the entry at once,
    /// no  evaluations are done again
    Till {
        #[arg(value_name = "TILL")]
        till: String,

        #[command(subcommand)]
        from: Option<StartSubcommandB>,
    },
}

#[derive(Debug, Subcommand)]
pub enum StartSubcommandB {
    /// Can use this argument to start the task at some point of the day instead of the current
    /// system time
    From {
        #[arg(value_name = "FROM")]
        start: String,
    },
}
