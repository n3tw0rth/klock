use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "klock", version, about = "Track time from Jira/ClickUp boards to Jira Worklog/Clockify")]
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
    /// Unlink a board or clock integration from a project
    Remove {
        /// Project code (e.g. JP)
        project_code: Option<String>,
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
    /// Manage pending Clockify entries queued by `stop`
    Pending {
        #[command(subcommand)]
        action: Option<PendingAction>,
    },
}

#[derive(Subcommand)]
pub enum PendingAction {
    /// List pending entries (default if no action)
    List,
    /// Edit a pending entry by its index
    Edit {
        /// Pending entry index (see `pending list`)
        idx: u32,
        /// Override hours (decimal)
        #[arg(long)]
        hours: Option<f32>,
        /// Override start time (HHMM)
        #[arg(long)]
        start: Option<String>,
        /// Override end time (HHMM)
        #[arg(long)]
        end: Option<String>,
        /// Override description (the [KEY] prefix is added on push)
        #[arg(long)]
        description: Option<String>,
    },
    /// Remove a pending entry by its index
    Remove {
        idx: u32,
    },
    /// Push all pending entries to Clockify
    Push {
        /// Skip confirmation prompt
        #[arg(long, short)]
        yes: bool,
    },
}

#[derive(Subcommand)]
pub enum AuthAction {
    /// Log in to a board or clock integration
    Login,
    /// Log out from a board or clock integration
    Logout,
}
