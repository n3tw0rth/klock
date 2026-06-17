# Jired — Claude Code Build Instructions

Rust CLI tool for collecting tasks from boards (Jira, ClickUp) and logging time to clocks (Jira Worklog, Clockify).
Each section is a standalone build unit. Execute one at a time.

---

## Section 1 — Project Scaffold

**Goal:** Init Rust project, define workspace layout, add all dependencies.

### Tasks
1. Create new binary crate: `cargo new jired --bin`
2. Set up module folders:
```
src/
  main.rs
  cli/
    mod.rs
  commands/
    mod.rs
    auth.rs
    add.rs
    start.rs
    stop.rs
    set.rs
    log.rs
  config/
    mod.rs
    models.rs
  boards/
    mod.rs
    jira.rs
    clickup.rs
  clocks/
    mod.rs
    jira.rs
    clockify.rs
  session/
    mod.rs
  error.rs
```
3. Create stub `mod.rs` files with `todo!()` placeholders in each module.

### `Cargo.toml` Dependencies
```toml
[dependencies]
clap = { version = "4", features = ["derive"] }
tokio = { version = "1", features = ["full"] }
reqwest = { version = "0.12", features = ["json"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
toml = "0.8"
inquire = "0.7"
anyhow = "1"
thiserror = "1"
chrono = { version = "0.4", features = ["serde"] }
keyring = "2"
fuzzy-matcher = "0.3"
dirs = "5"
tokio-test = "0.4"
```

### Verification
- `cargo build` compiles with zero errors (stubs only)
- All modules are declared and reachable from `main.rs`

---

## Section 2 — Error & Config Models

**Goal:** Define all data structures and custom error types. No logic yet.

### Tasks

#### `src/error.rs`
Define `JiredError` using `thiserror`:
- `AuthError(String)`
- `ConfigError(String)`
- `NetworkError(String)`
- `PlatformError(String)`
- `SessionError(String)`
- `NotFound(String)`

Alias `pub type Result<T> = std::result::Result<T, JiredError>;`

#### `src/config/models.rs`
Define structs with `serde` Serialize/Deserialize:

```rust
// Platform types
pub enum BoardPlatform { Jira, ClickUp }
pub enum ClockPlatform { Jira, Clockify }

// Stored per integration
pub struct BoardConfig {
    pub id: String,           // user-assigned alias e.g. "jira-prod"
    pub platform: BoardPlatform,
    pub base_url: String,
    pub email: String,        // used for keyring lookup
}

pub struct ClockConfig {
    pub id: String,
    pub platform: ClockPlatform,
    pub base_url: String,
    pub email: String,
}

// Linked to a project code
pub struct ProjectConfig {
    pub code: String,         // e.g. "JP"
    pub board_ids: Vec<String>,
    pub clock_ids: Vec<String>,
    pub platform_project_id: String,   // remote ID on the board platform
    pub platform_project_name: String,
}

// Root config file
pub struct AppConfig {
    pub boards: Vec<BoardConfig>,
    pub clocks: Vec<ClockConfig>,
    pub projects: Vec<ProjectConfig>,
}

// Active session (persisted to ~/.jired/session.toml)
pub struct Session {
    pub project_code: String,
    pub task_id: String,
    pub task_title: String,
    pub board_id: String,
    pub started_at: chrono::DateTime<chrono::Local>,
    pub start_time_override: Option<String>,  // HHMM
    pub end_time_override: Option<String>,
    pub active_date: chrono::NaiveDate,
}
```

#### `src/config/mod.rs`
Stub functions (signatures only, `todo!()` body):
```rust
pub fn config_path() -> PathBuf
pub fn session_path() -> PathBuf
pub fn load_config() -> Result<AppConfig>
pub fn save_config(config: &AppConfig) -> Result<()>
pub fn load_session() -> Result<Option<Session>>
pub fn save_session(session: &Session) -> Result<()>
pub fn clear_session() -> Result<()>
```

### Verification
- `cargo check` passes
- All structs implement `Debug`, `Clone`, `Serialize`, `Deserialize`

---

## Section 3 — Config File I/O & Keyring

**Goal:** Implement config read/write from `~/.jired/config.toml` and credential storage.

### Tasks

#### `src/config/mod.rs` — Implement all stubs

- `config_path()` → `dirs::home_dir()/.jired/config.toml`
- `session_path()` → `dirs::home_dir()/.jired/session.toml`
- `load_config()` → read file → `toml::from_str`. If missing, return empty `AppConfig`.
- `save_config()` → `toml::to_string_pretty` → write file. Create dir if not exists.
- `load_session()` / `save_session()` / `clear_session()` → same pattern, `session.toml`

#### `src/config/mod.rs` — Credential helpers
```rust
pub fn store_credential(service: &str, account: &str, secret: &str) -> Result<()>
pub fn get_credential(service: &str, account: &str) -> Result<String>
pub fn delete_credential(service: &str, account: &str) -> Result<()>
```
- Use `keyring` crate. Service name format: `jired-{integration_id}`
- On `get_credential` not found → return `JiredError::AuthError`

### Config File Format (TOML)
```toml
[[boards]]
id = "jira-prod"
platform = "jira"
base_url = "https://org.atlassian.net"
email = "user@example.com"

[[clocks]]
id = "clockify-main"
platform = "clockify"
base_url = "https://api.clockify.me"
email = "user@example.com"

[[projects]]
code = "JP"
board_ids = ["jira-prod"]
clock_ids = ["clockify-main"]
platform_project_id = "10001"
platform_project_name = "Jira Project Alpha"
```

### Verification
- Write unit test: save config → reload → assert fields match
- Write unit test: store credential → retrieve → matches

---

## Section 4 — Board Traits & Jira Implementation

**Goal:** Define `Board` trait, implement Jira board client.

### Tasks

#### `src/boards/mod.rs` — Trait definition
```rust
pub struct RemoteProject {
    pub id: String,
    pub name: String,
    pub key: String,
}

pub struct RemoteTask {
    pub id: String,
    pub key: String,
    pub title: String,
    pub status: String,
    pub assignee: Option<String>,
}

#[async_trait::async_trait]
pub trait Board {
    async fn search_projects(&self, query: &str) -> Result<Vec<RemoteProject>>;
    async fn search_tasks(&self, project_id: &str, query: &str) -> Result<Vec<RemoteTask>>;
    fn platform_name(&self) -> &str;
}
```

Add `async-trait = "0.1"` to `Cargo.toml`.

#### `src/boards/jira.rs` — `JiraBoard` struct
Fields: `base_url`, `email`, `api_token` (fetched from keyring on construction).

Implement `Board` trait:
- `search_projects`: `GET /rest/api/3/project/search?query={q}` → parse `values[]`
- `search_tasks`: `GET /rest/api/3/search?jql=project={id}+AND+text~"{q}"` → parse `issues[]`
- Auth: Basic auth header (`email:api_token` base64)

#### `src/boards/clickup.rs` — `ClickUpBoard` struct
Fields: `api_token`, `team_id`.

Implement `Board` trait:
- `search_projects`: `GET https://api.clickup.com/api/v2/team/{team_id}/space` → parse spaces
- `search_tasks`: `GET /api/v2/team/{team_id}/task?query={q}` → parse tasks
- Auth: `Authorization: {api_token}` header

#### Factory function in `src/boards/mod.rs`
```rust
pub fn build_board(config: &BoardConfig) -> Result<Box<dyn Board>>
// Match platform → construct appropriate impl, load cred from keyring
```

### Verification
- Unit test `search_projects` with mocked HTTP (use `wiremock` or just test parsing logic)
- `cargo check` passes

---

## Section 5 — Clock Traits & Implementations

**Goal:** Define `Clock` trait, implement Jira Worklog and Clockify.

### Tasks

#### `src/clocks/mod.rs` — Trait definition
```rust
pub struct TimeEntry {
    pub task_id: String,
    pub hours: f32,
    pub description: String,
    pub date: chrono::NaiveDate,
    pub start_time: Option<String>,   // HHMM
    pub end_time: Option<String>,
}

#[async_trait::async_trait]
pub trait Clock {
    async fn log_time(&self, entry: &TimeEntry) -> Result<()>;
    async fn get_logged_time(&self, task_id: &str, date: chrono::NaiveDate) -> Result<f32>;
    fn platform_name(&self) -> &str;
}
```

#### `src/clocks/jira.rs` — `JiraClock` struct
Implement `Clock`:
- `log_time`: `POST /rest/api/3/issue/{task_id}/worklog` with `timeSpentSeconds`, `started`
- `get_logged_time`: `GET /rest/api/3/issue/{task_id}/worklog` → sum entries matching date
- Auth: Basic auth

#### `src/clocks/clockify.rs` — `ClockifyClock` struct
Fields: `api_key`, `workspace_id`.

Implement `Clock`:
- `log_time`: `POST /api/v1/workspaces/{workspace_id}/time-entries` with ISO8601 start/end
- `get_logged_time`: `GET /api/v1/workspaces/{workspace_id}/user/{user_id}/time-entries` → filter by date
- Auth: `X-Api-Key: {api_key}` header

#### Factory function in `src/clocks/mod.rs`
```rust
pub fn build_clock(config: &ClockConfig) -> Result<Box<dyn Clock>>
```

### Verification
- `cargo check` passes
- Test `TimeEntry` serialization to correct JSON shape per platform

---

## Section 6 — Fuzzy Search & Interactive Prompts Helper

**Goal:** Centralize fuzzy search logic and all `inquire` prompt wrappers.

### Tasks

#### `src/session/mod.rs` — Fuzzy search helper
```rust
pub fn fuzzy_select<T>(
    items: Vec<T>,
    label_fn: impl Fn(&T) -> String,
    prompt: &str,
) -> Result<T>
// Use fuzzy-matcher to score items against user input
// If 1 result → inquire Confirm (y/n)
// If multiple → inquire Select with filtered list
// If 0 → return JiredError::NotFound
```

#### `src/session/mod.rs` — Prompt helpers
```rust
pub fn prompt_text(label: &str, default: Option<&str>) -> Result<String>
pub fn prompt_password(label: &str) -> Result<String>
pub fn prompt_select(label: &str, options: Vec<String>) -> Result<String>
pub fn prompt_confirm(label: &str) -> Result<bool>
pub fn prompt_date(label: &str) -> Result<chrono::NaiveDate>
pub fn prompt_time(label: &str, default: Option<&str>) -> Result<String>
// prompt_time: Text prompt, validate format HHMM, reprompt on invalid
```

All functions use `inquire` internally. Wrap inquire errors into `JiredError`.

### Verification
- `cargo check` passes
- Manual test: run fuzzy_select with sample vec, verify selection works

---

## Section 7 — CLI Definition (clap)

**Goal:** Define all commands, subcommands, and args with `clap` derive. No logic yet.

### Tasks

#### `src/cli/mod.rs`
```rust
#[derive(Parser)]
#[command(name = "jired", version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Auth { #[command(subcommand)] action: AuthAction },
    Add {
        project_code: Option<String>,
        search_string: Option<String>,
    },
    Start {
        project_code: Option<String>,
        search_string: Option<String>,
        #[arg(long)] start: Option<String>,
        #[arg(long)] end: Option<String>,
    },
    Stop {
        #[arg(long)] at: Option<String>,
    },
    Set {
        date: Option<String>,
    },
    Log {
        #[arg(long)] summary: bool,
    },
}

#[derive(Subcommand)]
pub enum AuthAction {
    Login,
    Logout,
}
```

#### `src/main.rs`
```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Auth { action } => commands::auth::handle(action).await?,
        Commands::Add { .. } => commands::add::handle(...).await?,
        Commands::Start { .. } => commands::start::handle(...).await?,
        Commands::Stop { at } => commands::stop::handle(at).await?,
        Commands::Set { date } => commands::set::handle(date).await?,
        Commands::Log { summary } => commands::log::handle(summary).await?,
    }
    Ok(())
}
```

### Verification
- `jired --help` shows all commands
- `jired auth --help`, `jired start --help` show correct args
- `cargo check` passes

---

## Section 8 — `auth` Command

**Goal:** Implement `auth login` and `auth logout` with full interactive fallback.

### Tasks

#### `src/commands/auth.rs`

**`login` flow:**
1. `prompt_select("Login to:", ["Board", "Clock"])`
2. If Board → `prompt_select("Platform:", ["Jira", "ClickUp"])`
3. If Clock → `prompt_select("Platform:", ["Jira", "Clockify"])`
4. Collect `base_url` (not needed for Clockify), `email`, `api_token` via prompts
5. Check config: if integration id already exists → notify, ask "Add another? (y/n)"
6. Auto-generate `id` as `{platform}-{index}` e.g. `jira-1`
7. Store token via `store_credential`
8. Append `BoardConfig` or `ClockConfig` to config → `save_config`
9. Print confirmation

**`logout` flow:**
1. `prompt_select("Logout from:", ["Board", "Clock"])`
2. List existing integrations of that type from config
3. If 1 → `prompt_confirm("Logout from {id}?")`
4. If multiple → `prompt_select` to pick one
5. `delete_credential`, remove from config → `save_config`

### Verification
- `jired auth login` → full prompt flow
- `jired auth logout` → correct removal from config
- Config file updated correctly after both

---

## Section 9 — `add` Command

**Goal:** Implement project-platform linking with fuzzy project search.

### Tasks

#### `src/commands/add.rs`

**Flow:**
1. `project_code` missing → `prompt_text("Project code:")`
2. `prompt_select("Integration type:", ["Board", "Clock"])`
3. List available integrations of that type from config → `prompt_select`
4. `search_string` missing → `prompt_text("Search:")`
5. Call `board.search_projects(search_string)` or equivalent
6. `fuzzy_select` on results
7. If project_code already in config → append board/clock id to existing `ProjectConfig`
8. If new → create `ProjectConfig` with selected board/clock, `platform_project_id`, name
9. `save_config`
10. Print: `Added {platform_project_name} → project {project_code}`

### Verification
- `jired add` → full prompt flow
- `jired add JP "alpha"` → skips those prompts, still prompts for integration type
- Config file reflects new project entry

---

## Section 10 — `start` Command

**Goal:** Begin a tracked session, resolve task via fuzzy search.

### Tasks

#### `src/commands/start.rs`

**Flow:**
1. Check `load_session()` → if active session exists → error: "Session already active. Run `jired stop` first."
2. `project_code` missing → `prompt_text("Project code:")`
3. Look up `ProjectConfig` by code → error if not found
4. `search_string` missing → `prompt_text("Search tasks:")`
5. Build board client via `build_board`
6. `board.search_tasks(platform_project_id, search_string)`
7. `fuzzy_select` on results
8. `start` missing → `prompt_time("Start time (HHMM):", Some("now"))`  
   `end` missing → `prompt_time("End time (HHMM):", None)` (optional, Enter to skip)
9. Build `Session` → `save_session`
10. Print: `Started [JP-101] Fix null pointer — 09:00`

### Verification
- `jired start` → full prompt flow
- `jired start JP "bug" --start 0900` → only prompts for missing fields
- Session file written to `~/.jired/session.toml`

---

## Section 11 — `stop` Command

**Goal:** End session, compute duration, log to all configured clocks.

### Tasks

#### `src/commands/stop.rs`

**Flow:**
1. `load_session()` → error if no active session
2. `at` missing → `prompt_time("Stop time (HHMM):", Some("now"))`
3. Compute hours from `start_time_override` or `started_at` → stop time
4. Load `ProjectConfig` for session's `project_code`
5. For each `clock_id` in project:
   a. `build_clock(clock_config)`
   b. Build `TimeEntry` with task_id, hours, description = task_title, date = active_date
   c. `clock.log_time(&entry)` 
   d. Print: `Logged {hours}h to {clock_id}`
6. `clear_session()`
7. Print summary

**Multiple clocks:** log to all automatically. No selection needed unless project has 0 clocks → error.

### Verification
- `jired stop` with active session → logs time, clears session
- `jired stop --at 1700` → uses provided time
- `jired stop` with no session → clear error message

---

## Section 12 — `set` & `log` Commands

**Goal:** Implement date override and time log summary.

### Tasks

#### `src/commands/set.rs`
1. `date` missing → `prompt_date("Date:")`
2. Parse `YYYY-MM-DD` → `chrono::NaiveDate`
3. Load session if exists → update `active_date` → save
4. If no session → store date in separate `~/.jired/state.toml` as `active_date`
5. Print: `Active date set to 2025-10-10`

#### `src/commands/log.rs`

**`--summary` flag:**
1. Load config → load all projects
2. For each project → for each clock → `get_logged_time(task_id, active_date)`
3. Group and display:
```
Date: 2025-10-10
─────────────────────────────
[JP-101] Fix null pointer
  jira-clock    2.5h
  clockify      2.5h

Total: 5.0h
```

Without `--summary`: show current active session (if any) with elapsed time.

#### `src/config/mod.rs` — Add state helpers
```rust
pub fn load_active_date() -> Result<chrono::NaiveDate>  // default: today
pub fn save_active_date(date: chrono::NaiveDate) -> Result<()>
```

### Verification
- `jired set 2025-10-10` → updates state
- `jired set` → prompt appears
- `jired log --summary` → shows time entries per clock
- `jired log` → shows active session or "No active session"

---

## Section 13 — Polish & Error UX

**Goal:** Consistent error messages, help text, edge cases.

### Tasks

1. **All commands:** wrap execution in `match` on `JiredError` variants → print user-friendly messages (no raw panics)
2. **Auth missing:** any command that needs board/clock credentials → detect → print "Run `jired auth login` first"
3. **No projects:** `start`/`add` with no boards configured → clear message
4. **Network errors:** retry once silently, then surface error with platform name
5. **`clap` help strings:** add `.about("...")` to every command and arg
6. **Colored output:** use `anstyle` (bundled with clap) or `colored` crate for status indicators
   - Green: success
   - Yellow: prompt / info
   - Red: error
7. **`--version`:** auto-generated by clap from `Cargo.toml`

### Verification
- `jired start` with no auth → helpful error, not panic
- `jired stop` with no session → helpful error
- All `--help` outputs are readable and complete
- `cargo clippy` passes with no warnings

---

## Build Order Summary

| # | Section | Depends On |
|---|---------|-----------|
| 1 | Scaffold | — |
| 2 | Error & Models | 1 |
| 3 | Config I/O & Keyring | 2 |
| 4 | Board Trait & Jira/ClickUp | 3 |
| 5 | Clock Trait & Jira/Clockify | 3 |
| 6 | Fuzzy Search & Prompts | 2 |
| 7 | CLI Definition (clap) | 2 |
| 8 | `auth` command | 3, 6, 7 |
| 9 | `add` command | 4, 6, 7 |
| 10 | `start` command | 4, 6, 7 |
| 11 | `stop` command | 5, 7 |
| 12 | `set` & `log` commands | 3, 5, 7 |
| 13 | Polish & Error UX | all |
