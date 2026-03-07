---
name: rust-maintenance
description: Maintain Rust code quality by running linters, formatters, and unit tests. Use this skill for ALMOST ANY modification to the Rust core, compiler errors, or when the user mentions "Zero-Fat" or "Iron Rules". Be proactive: if the code looks messy or tests are missing, trigger this skill to run `task lint` and `task test`. It is essential for ensuring type safety and code cleanliness in the VLog ecosystem.
allowed-tools:
  - "Bash(task *)"
  - Read
---

# Rust Maintenance Skill

A skill focused on maintaining a clean, efficient, and robust Rust codebase for the VLog project. It strictly adheres to the "Zero-Fat" coding principles and the "Iron Rules" of Rust development.

## Core Workflows

### 1. Code Quality Audit
Run standard checks to identify potential issues or non-compliant code.
- **cmd**: `task lint` (runs `cargo clippy --fix` and `cargo fmt`)
- Analyze Clippy warnings and apply manual fixes for complex architectural issues.

### 2. Verification
Ensure changes haven't introduced regressions.
- **cmd**: `task test`
- If tests fail, use the `Success Path Only` principle to refactor logic.

### 3. Cleanup
Remove build artifacts and temporary caches.
- **cmd**: `task clean`

## Guidelines
- **Zero-Fat Policy**: No unused code, no dead branches, and no opaque error swallowing.
- **Type Safety**: Prefer strict Rust types and typed `serde` models. Avoid `serde_json::Value` where typed structs are possible.
- **Task-First**: Always use `Taskfile` commands to ensure consistent environment variables and flags.
- **Documentation**: Document architecture in `docs/` or `AGENTS.md` in Japanese; keep code itself silent.

## Examples
- "コードをきれいに整えて" -> `task lint`
- "テストを実行して、壊れていないか確認して" -> `task test`
- "ビルドキャッシュをクリアして" -> `task clean`
- "新しいモジュールを追加したので品質チェックをお願い" -> `task lint` -> [Manual fix for new warnings]
