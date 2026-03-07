---
name: vlog-manager
description: Manage VLog operations including audio recording processing, Supabase data synchronization, and system monitoring. Use this skill whenever the user mentions processing wav/flac files, checking monitor health, or syncing data, even if they don't explicitly ask for a "task". It is CRITICAL for daily maintenance, handling Taskfile commands like `task dev`, `task process`, and `task sync`. ALWAYS trigger this for any operation involving the VLog backend or recording data.
allowed-tools:
  - "Bash(task *)"
  - Read
disable-model-invocation: true
argument-hint: "[filename]"
---

# VLog Manager Skill

A skill to assist with the daily operations of the VLog (VRChat Auto-Diary) system. It ensures that the Rust-based core pipeline and the frontend are running correctly and that data is processed efficiently.

## Core Operations

### 1. Audio Processing
Use this when new recordings are available in `data/recordings/`.
- **Single file**: `task process FILE=path/to/file.wav`
- **All pending**: `task process:all`
- **Today's recordings**: `task process:today`

### 2. System Monitoring
Use this to ensure the recording capture is active.
- **Start monitor**: `task dev`
- **Check status**: `task status` or `task service:status`
- **View logs**: `task logs`

### 3. Data Synchronization
Use this to push local summaries and transcripts to the Supabase backend.
- **Normal sync**: `task sync`
- **Full resync**: `task sync:full`

### 4. Web UI Management
- **Start dev server**: `task web:dev`
- **Deploy to Vercel**: `task web:deploy`

## Guidelines
- ALWAYS prefer `task` commands over raw `cargo` or `npm` commands.
- If invoked with `$ARGUMENTS`, treat it as a filename and run `task process FILE=data/recordings/$ARGUMENTS` after existence check.
- Before processing, check if the file exists in `data/recordings/`.
- If a process fails, check the logs using `task logs` to identify the issue.
- Maintain the "Success Path Only" principle by ensuring preconditions (like `FILE` argument) are met.

## Examples
- "昨日の録音を全部処理して" -> `task process:yesterday`
- "サーバーの状態はどう？" -> `task status`
- "データをSupabaseに同期して" -> `task sync`
- "新しく録音した `audio123.wav` をプロセシングして" -> `task process FILE=data/recordings/audio123.wav`
