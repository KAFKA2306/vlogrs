---
description: コンテンツ品質改善とコードベース最適化ワークフロー
---

# Agentic Optimization Workflow

// turbo-all

## 1. Content Coverage Analysis

- List all dates in `data/recordings/` and build a date inventory.
- Cross-reference against `data/summaries/`, `data/novels/`, `data/photos/` to identify coverage gaps per date.
- Report missing artifacts by date as a gap table.

## 2. Quality Audit

- Run `task curator:eval date=YYYYMMDD` on recent dates to evaluate content quality scores.
- Spot-check novel files for truncation or empty content: `find data/novels -size 0 -o -size -100c`.
- Spot-check photo files for zero-byte images: `find data/photos -size 0`.

## 3. Backfill Generation

- For each date with missing novels: `task novel:build date=YYYYMMDD`.
- For each date with missing photos: `task photo novel=data/novels/YYYYMMDD.md`.
- Run `task photos:fill` to batch-detect and generate remaining missing photos.

## 4. Codebase Optimization

- Run `task lint` and fix any reported issues.
- Identify unused imports or dead code in `src/` with `uv run ruff check src --select F401,F841`.
- Review `src/` file sizes — flag any file exceeding 200 lines for potential decomposition.

## 5. Sync & Commit

- Run `task sync` to push all new/updated content to Supabase.
- Run `task commit MESSAGE="optimization: [summary]"` to persist changes.
