# Jired

A command-line tool for logging time across multiple time tracking platforms including Jira, Clockify, and more.


## Overview

Terminal Time Tracker is a flexible, plugin-based CLI application that allows developers to log time directly from their terminal. The application provides a seamless experience for tracking time spent on tasks, regardless of which time tracking service your organization uses.

**Status: 🚧 Currently in active development**

## Features

- Simple command-line interface for time tracking
- multiple time logging providers:
  - Jira
  - Clockify
  - _More coming soon_
- fuzzy search tasks
- natural language like arguments
- report generation

## Installation

```bash
# Wait!,The project is still in development
```

## Quick Start

Once installed, you can begin tracking time with the following commands:

```bash
# Time stamp the start of a last task
jj start <project-code> <fuzzy-text>

# Time stamp the end of a task
jj stop

# Task with start and end time
jj start <project-code> <fuzzy-text> from 1800 till 1935 // end time is 07:35 PM

if from is not specified, the start time is set to the current time
if from is specified, but till is not, the end time will not be set


deafult scope is set to "today" and any time log will be added to the current day
the scope can be changed to any date using the following syntax
jj set <data> // data format is yyyy-mm-dd
```

## Provider Plugins (WIP)

- **Jira**: Log work directly to Jira issues
- **Clockify**: Track time in Clockify

-- providers can be combined, logging to multiple providers at once

## Configuration

Create a configuration file at `~/.config/ttrack/config.toml`:

```toml
not yet defined
```
