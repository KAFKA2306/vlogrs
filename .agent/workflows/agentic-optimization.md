---
description: Context7-Driven Content Quality & Codebase Optimization
---

# Agentic Optimization Workflow (Context7 Elite Edition)

// turbo-all

## [Layer 1] Runtime Integrity (Hard Foundation)
Strictest validation of the execution environment to prevent "phantom bugs".

1. **Verify Rust Environment**:
   - `rustc --version` must be 1.75+.
   - `cargo check --locked` must pass with zero errors.
2. **Verify Python Environment**:
   - `uv run python --version` must be 3.12+.
   - `uv sync --locked` to ensure no dependency drift.

## [Layer 2] Secret & Configuration Hardening
Zero-drift configuration management.

1. **Secret Entropy Audit**:
   - Check if `GOOGLE_API_KEY` exists and has a valid pattern (not `your_api_key`).
   - `grep -E "sk-|AIza" .env || echo "WARNING: Critical API Keys missing or invalid format"`
2. **Configuration Parity**:
   - Cross-reference `.env` with `src/infrastructure/settings.rs` to ensure all `std::env::var` calls are mapped.
   - Cross-reference `data/config.yaml` with `ProcessSettings` struct.

## [Layer 3] Data Inventory & Ghost Auditing
Physical tracking of project assets.

1. **The Inventory Matrix**:
   - Generate a date inventory from `data/recordings/` (WAV/FLAC).
   - Cross-reference with `data/summaries/` (TXT) and `data/novels/` (MD).
   - **Strict Rule**: Every summary MUST have a corresponding novel or be flagged as "Stalled Pipeline".
2. **Ghost Purge**:
   - `find data/ -type f -size 0 -delete` (Auto-delete zero-byte failures).
   - `find data/recordings -name "*.tmp" -mmin +60 -delete` (Cleanup stalled recordings).

## [Layer 4] Content Fidelity & Narrative Audit
Ensuring the "Soul" of the project remains high-quality.

1. **Novel Length Audit**:
   - `find data/novels -name "*.md" -size -500c` (Identify "thin" chapters).
2. **The Curator Evaluation**:
   - Run `task curator:eval date=YYYYMMDD` for the most recent 3 active dates.
   - **Threshold**: Reliability score < 0.8 requires mandatory manual review.
3. **Visual Integrity**:
   - `identify data/photos/*.png` (Ensure valid image headers and correct resolution).

## [Layer 5] Architectural Purity (Iron Rules)
Code is a liability; keep it lean and clean.

1. **The 200-Line Ceiling**:
   - `find src -name "*.rs" -o -name "*.py" | xargs wc -l | awk '$1 > 200 {print $2 " is OVER LIMIT (" $1 " lines)"}'`
   - **Action**: Any file over 200 lines MUST be prioritized for decomposition in the next task.
2. **Clippy Zero-Tolerance**:
   - `cargo clippy --all-targets -- -D warnings`. Warnings are treated as compilation errors.
3. **Rust Atomic Purity**:
   - Ensure `src/domain/` has NO dependencies on `src/infrastructure/`.
   - `grep -r "infrastructure" src/domain && echo "ERROR: Domain leakage detected"`

## [Layer 6] Logic & Side-Effect Safety
1. **Unwrap Audit**:
   - `grep -r "unwrap()" src | grep ".rs"`
   - **Strict Rule**: `unwrap()` is forbidden in `use_cases/` and `infrastructure/`. Use `expect("Contextual message")` or `Result` handling.
2. **Test Baseline**:
   - `cargo test` must achieve 100% pass rate.

## [Layer 7] Knowledge & Vision Sync (The Truth)
1. **Manifest Alignment**:
   - Check if `AGENTS.md` correctly lists all current command-line tools found in `src/main.rs`.
   - Check if `README.md` reflects the current production status of the Rust migration.
2. **Mermaid Reality Sync**:
   - Audit `docs/diagrams/` against `src/infrastructure/system.py` and `src/main.rs` call flows.
3. **Atomic Sync & Commit**:
   - `task sync` to push metadata to the cloud.
   - `task commit MESSAGE="optimization: Context7 Full Audit [L1-L7] completed"`
