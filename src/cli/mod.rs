use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "jired", version, about = "Track time from Jira/ClickUp boards to Jira Worklog/Clockify")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Manage authentication for boards and clocks
    Auth {
        #[command(subcommand)]
        action: AuthAction,
    },
    /// Link a project to a board or clock integration
    Add {
        /// Project code (e.g. JP)
        project_code: Option<String>,
        /// Search string for filtering projects
        search_string: Option<String>,
    },
    /// Begin a tracked session on a task
    Start {
        /// Project code (e.g. JP)
        project_code: Option<String>,
        /// Search string for filtering tasks
        search_string: Option<String>,
        /// Override start time (HHMM)
        #[arg(long)]
        start: Option<String>,
        /// Override end time (HHMM)
        #[arg(long)]
        end: Option<String>,
    },
    /// End the current session and log time
    Stop {
        /// Stop time override (HHMM)
        #[arg(long)]
        at: Option<String>,
    },
    /// Set the active date for time logging
    Set {
        /// Date in YYYY-MM-DD format
        date: Option<String>,
    },
    /// Show time log or active session
    Log {
        /// Show summary of all logged time for the active date
        #[arg(long)]
        summary: bool,
    },
}

#[derive(Subcommand)]
pub enum AuthAction {
    /// Log in to a board or clock integration
    Login,
    /// Log out from a board or clock integration
    Logout,
}
