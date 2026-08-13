#!/usr/bin/env python3
"""Extract conservative Social Mirror candidates from saved text.

This tool never upgrades summary-derived prose into a verified direct quote.
Use --source-kind raw_transcript only for source text that is itself the saved
transcript. Output is JSONL for review/import; it does not mutate Supabase.
"""

from __future__ import annotations

import argparse
import json
import re
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Iterable

DATE_HEADING_RE = re.compile(
    r"(?:20\d{6})|(?:20\d{2}[/-](?:0?[1-9]|1[0-2])[/-](?:0?[1-9]|[12]\d|3[01]))"
)
QUOTE_RE = re.compile(r"「([^」\n]{1,160})」")
SOCIAL_SIGNAL_AFTER_RE = re.compile(
    r"(?:と)?(?:言われ|教えられ|指摘され|評され|評価され|褒められ|"
    r"思われ|見られ|扱われ)"
)
SENTENCE_RE = re.compile(r"[^。！？\n]+[。！？]?", re.MULTILINE)
SIGNAL_WINDOW = 48


@dataclass(frozen=True)
class Candidate:
    occurred_at: str | None
    speaker_label: str
    observation_type: str
    quote_text: str | None
    paraphrase: str | None
    source_excerpt: str
    source_kind: str
    quote_candidate: str | None
    speaker_confidence: float
    is_public: bool
    speaker_is_public: bool
    needs_raw_verification: bool


def normalize_date(token: str) -> str | None:
    parts = re.findall(r"\d+", token)
    if len(parts) == 1 and len(parts[0]) == 8:
        compact = parts[0]
        return f"{compact[:4]}-{compact[4:6]}-{compact[6:8]}"
    if len(parts) == 3 and len(parts[0]) == 4:
        year, month, day = (int(part) for part in parts)
        return f"{year:04d}-{month:02d}-{day:02d}"
    return None


def iter_candidates(text: str, source_kind: str) -> Iterable[Candidate]:
    current_date: str | None = None
    seen: set[tuple[str | None, str, str]] = set()

    for block in text.splitlines():
        date_match = DATE_HEADING_RE.search(block)
        if date_match:
            current_date = normalize_date(date_match.group(0))

        for match in SENTENCE_RE.finditer(block):
            sentence = match.group(0).strip()
            if not sentence:
                continue

            quote_matches = list(QUOTE_RE.finditer(sentence))
            for index, quote_match in enumerate(quote_matches):
                quote = quote_match.group(1)
                next_quote_start = (
                    quote_matches[index + 1].start()
                    if index + 1 < len(quote_matches)
                    else len(sentence)
                )
                after_quote = sentence[quote_match.end():next_quote_start][:SIGNAL_WINDOW]
                if not SOCIAL_SIGNAL_AFTER_RE.search(after_quote):
                    continue

                dedupe_key = (current_date, quote, sentence)
                if dedupe_key in seen:
                    continue
                seen.add(dedupe_key)

                if source_kind == "raw_transcript":
                    yield Candidate(
                        occurred_at=current_date,
                        speaker_label="unknown",
                        observation_type="direct_quote",
                        quote_text=quote,
                        paraphrase=None,
                        source_excerpt=sentence,
                        source_kind=source_kind,
                        quote_candidate=None,
                        speaker_confidence=0.0,
                        is_public=False,
                        speaker_is_public=False,
                        needs_raw_verification=False,
                    )
                else:
                    # A summary may contain wording invented or normalized by an LLM.
                    # Preserve it only as a review candidate until raw evidence exists.
                    yield Candidate(
                        occurred_at=current_date,
                        speaker_label="unknown",
                        observation_type="paraphrase",
                        quote_text=None,
                        paraphrase=sentence,
                        source_excerpt=sentence,
                        source_kind="summary_derived",
                        quote_candidate=quote,
                        speaker_confidence=0.0,
                        is_public=False,
                        speaker_is_public=False,
                        needs_raw_verification=True,
                    )


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("input", type=Path, help="Markdown/text export to scan")
    parser.add_argument(
        "--source-kind",
        choices=("summary_derived", "raw_transcript"),
        default="summary_derived",
        help="Evidence strength of the input text",
    )
    parser.add_argument("--output", type=Path, help="Write JSONL here instead of stdout")
    args = parser.parse_args()

    text = args.input.read_text(encoding="utf-8")
    lines = [
        json.dumps(asdict(candidate), ensure_ascii=False)
        for candidate in iter_candidates(text, args.source_kind)
    ]
    payload = "\n".join(lines)
    if payload:
        payload += "\n"

    if args.output:
        args.output.write_text(payload, encoding="utf-8")
    else:
        print(payload, end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
