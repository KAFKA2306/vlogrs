#!/usr/bin/env python3

import argparse
from pathlib import Path

from src.infrastructure.preprocessor import TranscriptPreprocessor




def main():
    parser = argparse.ArgumentParser(description="Preprocess transcript for LLM")
    parser.add_argument("infile", type=Path, help="Input file path")
    parser.add_argument("-o", "--outfile", type=Path, help="Output file path")
    parser.add_argument(
        "--remove-fillers", action="store_true", help="Remove common fillers"
    )
    parser.add_argument(
        "--merge-lines", action="store_true", help="Merge all lines into one paragraph"
    )
    parser.add_argument(
        "--mask-fillers", action="store_true", help="Replace fillers with [FILLER]"
    )
    parser.add_argument(
        "--aggressive", action="store_true", help="Aggressive filler removal"
    )
    parser.add_argument(
        "--dedupe", action="store_true", help="Deduplicate consecutive words"
    )

    args = parser.parse_args()

    if not args.infile.exists():
        print(f"Error: File {args.infile} not found.")
        return

    preprocessor = TranscriptPreprocessor()
    txt = args.infile.read_text(encoding="utf-8")

    if args.remove_fillers:
        txt = preprocessor.remove_fillers(
            txt, mask=args.mask_fillers, aggressive=args.aggressive
        )

    if args.dedupe:
        txt = preprocessor.dedupe_words(txt)

    if args.merge_lines:
        txt = preprocessor.merge_lines(txt)

    if args.outfile:
        args.outfile.write_text(txt, encoding="utf-8")
        print(f"Processed text saved to {args.outfile}")
    else:
        print(txt)


if __name__ == "__main__":
    main()
