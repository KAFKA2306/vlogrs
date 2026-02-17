---
description: 定期ヘルスチェックとパイプライン保守ワークフロー
---

# Agentic Management Workflow

// turbo-all

## 1. Health Survey

- Run `systemctl --user status vlog --no-pager --lines=10` to verify systemd unit state.
- Run `journalctl --user -n 50 -u vlog --no-pager` and scan for ERROR/CRITICAL lines.
- Run `task sync` to verify Supabase connectivity.

## 2. Processing Audit

- Run `task process:pending` to catch files missed by the auto-pipeline.
- Compare `data/recordings` vs `data/transcripts` vs `data/summaries` to ensure 1:1 mapping for each date.
- Run `ls data/novels/ | wc -l` and `ls data/photos/ | wc -l` to check novel/photo coverage.

## 3. Pipeline Repair

- Run `task repair` to execute the `PipelineRepairAgent` for automated gap-filling (transcripts → summaries → novels → photos).
- Verify repair results by re-running `task process:pending`.

## 4. Sync Verification

- Run `task sync` to push any repaired data to Supabase.
- Query Supabase via MCP to confirm row counts match local file counts.

## 5. Maintenance

- Run `task lint` to ensure code stability.
- Run `task clean` if `__pycache__` or `.ruff_cache` are growing.
- Run `task commit MESSAGE="maintenance: [summary]"` if any fixes were applied.
