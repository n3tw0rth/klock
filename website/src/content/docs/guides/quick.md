---
title: Quick Start
description: A guide in my new Starlight docs site.
---

You can add a alias to make using the cli easier. i suggest something `j`, but you shall create your own alias.

Once installed, you can begin tracking time with the following commands:


#### Add a new project
Running this command will show all the projects within the organization and you can save the selected project under the project code. This will update the configuration file.
```bash
j add <project-code> <search-text>  # if the project code is eg: EA-ACME-PROJECT, you can use `acme` as the key and add the search text to search thru the project codes
```

#### Start a task
This will log the current local timestamp. And this time stamp will be used when stoping the current task. No timers are used, just a simple time difference calculation at the task termination. 
```bash
j start <project-code> <search-text> # search for any in progress task. eg: to select a meeting tasks you can search for `mee` that will show you all the tasks contain `mee`. and select the meeting task.

eg: j start acme mee
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
