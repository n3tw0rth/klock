# Jired

[![CI](https://github.com/n3tw0rth/jired/actions/workflows/github-actions.yml/badge.svg)](https://github.com/n3tw0rth/jired/actions/workflows/github-actions.yml)

**Status: 🚧 Currently in active development**

A command-line tool for logging time across multiple time tracking platforms including Jira, Clockify, and more.

## Overview

Jired, is a CLI application that allows users to log time directly from their terminal. The application provides a seamless experience for tracking time spent on tasks, without the need of clicking a lots of buttons and dropdown on a web UI.


CLI consists of Boards and Clocks
- __Boards__: Jira, ClickUp are Boards. Places where you can create tickets.
- __Clocks__: Clockify, Jira(worklog) are Clocks, where you can log time spend on each ticket.


## Features

- Simple command-line interface for time tracking
- Natural language-like commands
- Multiple time logging providers:
  - Clockify
  - Jira _(work log)_
  - _and more_
- Fast, case-sensitive substring search for filtering entries- 

## Installation
For now linux users can install the CLI using the install script or may simply build the the project with `--release` flag and copy it to a preffered directory. I am currently working on supporting selected package managers on each OS.


## Quick Start

You can add a alias to make using the cli easier. i suggest something `j`, but you shall create your own alias.

Once installed, you can begin tracking time with the following commands:


#### Add a new project
Running this command will show all the projects within the organization and you can save the selected project under the project code. This will update the configuration file.
```bash
j add <project-code> <search-text>
```

#### Start a task
This will log the current local timestamp. And this time stamp will be used when stoping the current task. No timers are used, just a simple time difference calculation at the task termination. 
```bash
j start <project-code> <search-text>
```

#### End a task
An ongoing task can be stopped running this command. Any ongoing process will also terminated when you start a new task. Assuming someone might not work on two task simultaneously.
```bash
j stop
```

#### Task with start and end time
You will have the option to start a task with the end time and start time, or you can only pass the end time and CLI will mark the start time as the current local time. Notice that the end time comes first and then the start time. 
```bash
j start <project-code> <fuzzy-text> from 1800 till 1935 // end time is 07:35 PM
```

#### Set the date
CLI's default scope is set to "today" and any time log will be logged under the present day. The scope can be changed to any date using the following command. Setting environment variables from the CLI it self is complex, so the command will out put a platform specific command to create a environment variable. (The idea is to set it only to the current terminal)
```bash
j set <data> // data format is yyyy-mm-dd
```

#### Log time
Once you are ready to log the time for the day, you can run the following command to log time. This will log the time on all the clocks you've defined on the configuration file. 
```bash
j log
```


## Running with Docker

You can also try Jired through Docker:

```bash
# Build the image (if not already built)
docker build -t jired .

# Run the app
docker run -it --rm jired bash
$: jired help
```

Or use the prebuilt image from GHCR

```bash
docker run -it --rm ghcr.io/n3tw0rth/jired:main bash
$: jired help
```

## Configuration

Create a configuration file at `~/.config/ttrack/config.toml`:

 |Platform |  Example                                                    |
 | ------- |  ----------------------------------------                   |
 | Linux   |  /home/alice/.config/jired/config.toml                      |
 | macOS   |  /Users/Alice/Library/Application Support/jired/config.toml |
 | Windows |  C:\Users\Alice\AppData\Roaming\jired/config.toml           |

```toml
# .config/jired/config.toml
clocks = [
    "clockify",
]
editor = "nvim"

[[clockify_projects]]
code = ""
key = ""
id = ""
            
```
