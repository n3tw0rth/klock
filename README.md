# klock

A terminal time tracker that pulls tickets from your boards (Jira, ClickUp) and logs work to your clocks (Jira Worklog, Clockify).

## Install

```sh
cargo install --path .
```

## Quick start

```sh
# 1. Authenticate at least one board and one clock
klock auth login

# 2. Link a project to those integrations
klock add ACME

# 3. Start a session on a ticket
klock start ACME

# 4. End the session — entries are queued for review
klock stop --at 1700

# 5. Audit the queue and push to all linked clocks
klock pending
klock pending push
```

Each command runs interactively when arguments are omitted.

## Commands

| Command | Purpose |
|---|---|
| `klock auth login` / `logout` | Manage board and clock integrations |
| `klock add [CODE]` | Link a project to a board or clock |
| `klock remove [CODE]` | Unlink a board or clock from a project |
| `klock start [CODE] [QUERY]` | Search for a ticket and begin tracking |
| `klock stop [--at HHMM]` | End the session and queue the entry |
| `klock pending [list \| edit \| remove \| push]` | Audit and flush the queue |
| `klock log [--summary]` | Show the active session or daily totals |
| `klock set [YYYY-MM-DD]` | Set the active logging date |

## How it works

- A project links to one or more **boards** (Jira, ClickUp) for ticket search and one or more **clocks** (Jira Worklog, Clockify) for time logging.
- `klock stop` does not post anywhere — it queues a single pending entry per session, carrying the list of clocks it should fan out to.
- `klock pending push` writes the entry to every linked clock. Per-clock outcomes are tracked, so a partial failure stays in the queue and the next `push` skips the clocks that already succeeded.
- Clockify entries are written with a `[ISSUE-KEY] title` description and a `projectId` resolved by matching the Clockify project name to the Jira project — mirroring how the Clockify Jira marketplace app displays linked entries.
- When you add a Jira worklog clock to a project that already has a Jira board, klock reuses the board's API token. No second authentication.

## Storage

- `~/.klock/config.toml` — integrations and project links
- `~/.klock/session.toml` — active session
- `~/.klock/state.toml` — active logging date
- `~/.klock/pending.toml` — queued entries
- API tokens — OS keyring, service `klock-<integration-id>`
