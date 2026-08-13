# Social Mirror / People Said

## Purpose

KafLog already records what happened and how the user experienced it. Social Mirror adds a separate question:

> What did other people actually say about me at that time?

The purpose is not personality diagnosis. It is to preserve multiple, sometimes contradictory, versions of the user as observed by other people and to keep each observation traceable to evidence.

## Evidence levels

### `direct_quote`

Use only when the exact wording is present in a raw transcript or was entered manually as an exact quote.

- `quote_text` is required.
- Automated raw-transcript extraction may preserve the exact substring already present in the transcript.
- AI must not rewrite a sentence and store the rewrite as a direct quote.

### `paraphrase`

Use when the meaning is useful but exact wording is not verified.

- `paraphrase` is required.
- Backfill from AI-generated diary summaries always starts here.
- A `summary_derived` observation must not be silently promoted to raw-transcript verified.

### `inferred_impression`

Use for an interpretation such as “they seemed to see me as reliable.” This is explicitly not something the other person necessarily said.

- `paraphrase` stores the inference.
- The Reader labels it as inference.
- It must never be displayed as a quotation.

## Source strength

`source_kind` records how the observation was obtained.

1. `raw_transcript` — strongest automated evidence for exact wording.
2. `manual` — user-entered record; may be an exact quote or paraphrase depending on `observation_type`.
3. `summary_derived` — recovered from an existing generated diary; useful for backfill candidates but not proof of exact wording.

## Privacy

Social observations contain third-party speech, so privacy is opt-in.

- `is_public` defaults to `false`.
- `speaker_is_public` defaults to `false` independently.
- Public Reader queries only `is_public = true` rows.
- Public Reader does not request or render `source_excerpt`.
- A public observation with `speaker_is_public = false` renders the speaker as anonymous.

## Backfill workflow

Run the conservative candidate extractor against a saved Markdown/text export:

```bash
python scripts/backfill_social_observations.py data/export.md \
  --source-kind summary_derived \
  --output social-candidates.jsonl
```

The script is deliberately narrow. It looks for quoted text followed nearby by explicit interpersonal speech signals such as “言われた”, “教えられた”, “指摘された”, or “褒められた”. It deduplicates identical diary rows.

Summary-derived output remains `paraphrase`, preserves the quoted phrase only as `quote_candidate`, sets `needs_raw_verification = true`, and is private by default.

When scanning an actual raw transcript:

```bash
python scripts/backfill_social_observations.py data/transcript.txt \
  --source-kind raw_transcript \
  --output social-candidates.jsonl
```

Only then may an extracted phrase become `direct_quote` automatically.

## Non-goals

- Automatically deciding one stable personality for the user.
- Converting sentiment scores into relationship quality.
- Guessing speaker identity when diarization is uncertain.
- Publishing another person’s name or speech by default.
- Treating an AI diary summary as verbatim evidence.

## Reader behavior

`People Said` is a separate view beside Diary and Novel.

The card shows the observation and, when available, the user’s own reaction. Detail view shows evidence type and source strength. If the Social Mirror table is not yet deployed, Diary and Novel remain usable; Social Mirror failure does not take down the Reader.
