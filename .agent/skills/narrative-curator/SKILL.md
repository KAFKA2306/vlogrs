---
name: narrative-curator
description: Evaluate and improve the quality, tone, and consistency of generated narratives and audio summaries. ACTIVATE this skill whenever the user mentions "narrative", "diary", "novel", "summary", or asks about personae and content quality. Do not wait for an explicit request to audit; if the user is refining creative output or checking for logic gaps in logs, this skill MUST be used. It integrates deeply with `task curator:eval`.
allowed-tools:
  - "Bash(task *)"
  - Read
  - Edit
argument-hint: "[date YYYYMMDD]"
---

# Narrative Curator Skill

A skill dedicated to ensuring the highest quality of AI-generated content within the VLog system. It focuses on narrative flow, persona preservation, and character consistency.

## Core Responsibilities

### 1. Quality Evaluation
Run the automated evaluation suite for specific dates.
- **cmd**: `task curator:eval date=YYYYMMDD`
- Analyze the output of the evaluation (pass/fail metrics) and identify areas for manual refinement.

### 2. Style and Tone Audit
Manually review generated files in `data/novels/` to ensure they match the established "Auto-Diary" aesthetic.
- Check if the persona remains consistent across multiple chapters.
- Verify that summaries accurately reflect the core events of the recorded sessions.

### 3. Content Refinement
Suggest or apply edits to `data/novels/*.md` or `data/summaries/*.txt` when the automated evaluation flags issues.
- Re-run `task novel:build` if summaries are updated to regenerate the narrative.

## Guidelines
- Follow the "Silent Operator" principle: keep edits concise and focused on narrative quality.
- Use the `media-expert` skill (if available) as reference for tone and image generation prompts.
- When regenerating content, ensure `task sync` is run afterwards to update the backend.

## Examples
- "昨日の小説の整合性をチェックして" -> `task curator:eval date=$(date -d "yesterday" +%Y%m%d)`
- "要約が短すぎるので、もう少し詳しく書き直して" -> [Edit summary file] -> `task novel:build`
- "日記のトーンをもう少し優しくして" -> [Modify system prompt or edit existing files]
- "生成された画像が小説の内容と合っているか確認して" -> `task photos:fill` で不足画像を補完して確認。
