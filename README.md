<div align="center">

# ⏱ klock

**A terminal time tracker that pulls tickets from your boards and logs work to your clocks.**

[![Rust](https://img.shields.io/badge/rust-2021-orange?style=flat-square&logo=rust)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](LICENSE)
[![Version](https://img.shields.io/badge/version-0.1.0-lightgrey?style=flat-square)](Cargo.toml)
[![Boards](https://img.shields.io/badge/boards-Jira%20%C2%B7%20ClickUp-0052CC?style=flat-square&logo=jira)](#how-it-works)
[![Clocks](https://img.shields.io/badge/clocks-Worklog%20%C2%B7%20Clockify-03A9F4?style=flat-square)](#how-it-works)

</div>

---

## Install

```sh
cargo install --path .
```

## Quick start

```sh
klock auth login       # 1. Authenticate at least one board and one clock
klock add ACME         # 2. Link a project to those integrations
klock start ACME       # 3. Start a session on a ticket
klock stop --at 1700   # 4. End the session — entries are queued for review
klock pending          # 5. Audit the queue…
klock pending push     #    …and push to all linked clocks
```

> [!TIP]
> Each command runs interactively when arguments are omitted.

## Commands

| Command | Purpose |
|:--|:--|
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

| Path | Contents |
|:--|:--|
| `~/.klock/config.toml` | Integrations and project links |
| `~/.klock/session.toml` | Active session |
| `~/.klock/state.toml` | Active logging date |
| `~/.klock/pending.toml` | Queued entries |
| OS keyring | API tokens, service `klock-<integration-id>` |

## License

[MIT](LICENSE)
