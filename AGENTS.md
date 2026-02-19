# VRChat Auto-Diary

See [Global Configuration](file:///home/kafka/projects/.agent/AGENTS.md)

## Domain Overrides

### VRChat Process Monitoring
- Source of Truth: [constants.rs](file:///home/kafka/vlog/src/domain/constants.rs)
- Process names: `VRChat.exe`, `vrchat` (See `DEFAULT_PROCESS_NAMES`)
- Check interval: 5 seconds (See `MONITOR_CHECK_INTERVAL_DEFAULT`)

### Audio Pipeline
- **Source of Truth**: [constants.rs](file:///home/kafka/vlog/src/domain/constants.rs)
- All parameters (Sample Rate, Channels, Min Duration) MUST be retrieved from `constants.rs`.
- Standard: Capture at Hardware-Native (48kHz/Stereo) -> Process at AI-Target (16kHz/Mono).

### Supabase Schema
- `recordings`: session metadata
- `transcripts`: Whisper output
- `summaries`: Gemini-generated summaries

### Whisper Config
- Model: `large-v3`
- Device: See `WHISPER_DEVICE` in [constants.rs](file:///home/kafka/vlog/src/domain/constants.rs)
- VAD filter: See `WHISPER_VAD_FILTER` in [constants.rs](file:///home/kafka/vlog/src/domain/constants.rs)
- Language: See `WHISPER_LANGUAGE` in [constants.rs](file:///home/kafka/vlog/src/domain/constants.rs)

### Gemini Config
- **Model**: See `GEMINI_MODEL` in [constants.rs](file:///home/kafka/vlog/src/domain/constants.rs) (Primary)
- Refer to `docs/gemini.md` for the full model selection policy.

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




