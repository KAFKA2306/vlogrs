# VRChat Auto-Diary

See [Global Configuration](file:///home/kafka/projects/.agent/AGENTS.md)

## Domain Overrides

### VRChat Process Monitoring
- Source of Truth: [constants.rs](file:///home/kafka/vlog/src/domain/constants.rs)
- Process names: `VRChat.exe`, `vrchat` (See `DEFAULT_PROCESS_NAMES`)
- Check interval: 5 seconds (See `MONITOR_CHECK_INTERVAL_DEFAULT`)

### Audio Pipeline
- Source of Truth: [constants.rs](file:///home/kafka/vlog/src/domain/constants.rs)
- Sample rate: 16000 Hz (See `DEFAULT_SAMPLE_RATE`)
- Channels: 1 (See `DEFAULT_CHANNELS`)
- Minimum recording: 60 seconds (See `MIN_RECORDING_SECS_DEFAULT`)
- Priority: Transcription accuracy first (record directly in ASR target format)

### Supabase Schema
- `recordings`: session metadata
- `transcripts`: Whisper output
- `summaries`: Gemini-generated summaries

### Whisper Config
- Model: `large-v3`
- Device: `cuda` (fallback: `cpu`)
- VAD filter: enabled
- Language: `ja`

### Gemini Config
- Model: `gemini-3-flash`

## Commands

See `Taskfile.yaml` for all commands.

Key tasks:
- `task dev` - Auto-monitoring mode
- `task process FILE=audio.wav` - Process single recording
- `task sync` - Supabase sync
- `task web:dev` - Frontend dev server

## Agentic Management

See [.agent/workflows/agentic-management.md](file:///home/kafka/projects/vlog/.agent/workflows/agentic-management.md) for maintenance procedures.
See [.agent/workflows/agentic-optimization.md](file:///home/kafka/projects/vlog/.agent/workflows/agentic-optimization.md) for content quality improvement procedures.
Agents should perform a weekly health check and audit processed recordings.

## MCP Servers

- `supabase-mcp-server` - Database operations
- `netlify` - Frontend deployment




