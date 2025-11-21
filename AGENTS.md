# Repository Guidelines

Documents must be written by Japanese.
Response in Japanese to User.

create and update README.md in all directories in Japanese

## Project Structure & Module Organization

The project is source-first: `src/main.py` launches `Application`, which orchestrates the process monitor and recorder. Long-running orchestration lives in `src/app.py`; reusable infrastructure (settings, audio capture, Whisper-based transcription, summarization, process monitoring) sits in `src/infrastructure/`. Domain DTOs live in `src/domain/`, while executable workflows reside under `src/services/`.

CLI entry point `src/cli.py` provides individual commands (`check`, `record`, `transcribe`, `summarize`, `write`, `process`) for manual execution and development workflows.

Configuration is split between:

- `.env` – API keys and secrets (Git-ignored)
- `config.yaml` – all other settings (process monitoring, audio, Whisper, Gemini, retention policy)

Runtime artifacts such as `recordings/`, `transcripts/`, and `diaries/` are created alongside the repo root and should be kept out of version control thanks to `.gitignore`. Add future tests under a top-level `tests/` package mirroring `src/`.

## Build, Test, and Development Commands

### Taskfile Commands (推奨)

```bash
task              # クイックガイド表示
task setup        # 依存同期 (uv sync)
task dev          # フォアグラウンド実行
task lint         # Ruffチェック＋フォーマット
task clean        # キャッシュクリア
```

### Service Management

```bash
task up           # systemdサービス有効化・起動
task down         # systemdサービス無効化・停止
task status       # サービス状態確認
task logs         # サービスログ追尾
task restart      # サービス再起動
```

### Recording & Processing

```bash
task record                              # 手動録音
task process FILE=path/to/audio.wav      # 一括処理（文字起こし→要約→日記）
task transcribe FILE=path/to/audio.wav   # 文字起こしのみ
task summarize FILE=path/to/text.txt     # 要約のみ
task write FILE=path/to/summary.txt      # 日記出力のみ
```

### Raw Commands

```bash
uv sync                       # 依存同期
uv run python -m src.main     # メイン実行
uv run python -m src.cli --help  # CLIヘルプ
uv run ruff check src         # リント
uv run ruff format src        # フォーマット
```

## Coding Style & Naming Conventions

Code targets Python 3.11 with type hints and dataclasses where helpful (`src/domain/entities.py`). Keep lines ≤88 chars (see `[tool.ruff]`), use four-space indentation, snake_case filenames, and noun-based class names. Public functions should describe their action (`get_latest_recording`), and long-running workflows deserve short docstrings or inline comments explaining sequencing.

Configuration values belong in `config.yaml` or `.env`, not hardcoded in source files. Load `.env` via `python-dotenv` and `config.yaml` via `PyYAML`.

## Commit & Pull Request Guidelines

Commits follow short, imperative messages (`Add initial vlog app files`, `Add Python gitignore`). Keep scope focused, reference issues in the body when available, and avoid bundling unrelated formatting with feature work. PRs should include: summary of changes, manual/automated test evidence (`task lint`, `uv run pytest` output), configuration notes (e.g., new settings), and screenshots or diary samples if user-facing behavior changed. Request reviewers only after lint/tests pass and large assets are excluded from git history.
