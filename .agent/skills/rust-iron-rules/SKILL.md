---
name: rust-iron-rules
description: Rust engineering rules for architecture, type design, logging, and clippy policy alignment. Trigger this skill whenever users request refactors, coding standards decisions, module boundary changes, or strict Rust style enforcement, because inconsistent rules create review churn and unstable behavior across the codebase.
user-invocable: false
allowed-tools:
  - Read
---

# Rust Iron Rules Skill

## 1. Zero-Fat Code Discipline
- Keep code direct and minimal.
- Remove dead code immediately.
- Keep function complexity low and split oversized files when needed.

## 2. Failure Semantics
- Make failure paths explicit.
- Avoid silent error swallowing.
- Use contextual failures where needed to keep root cause visible.

## 3. Architecture Boundaries
- Maintain clear layering in `domain`, `use_cases`, and `infrastructure`.
- Prevent dependency inversion violations and cross-layer leakage.

## 4. Quality and Logging
- Prefer structured logging via `tracing`.
- Enforce clippy and formatting consistency before merge.

## 5. Type Safety
- Prefer strong domain types over primitive obsession.
- Use typed models instead of unstructured payload handling whenever possible.
