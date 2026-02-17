# src

VLog Auto Diaryのメインソースコード。Rustへの移行進行中（Clean Architectureパターン）。

## ディレクトリ構成

- `domain/` - エンティティとインターフェース
- `use_cases/` - ビジネスロジック
- `infrastructure/` - 外部依存実装（API, Audio, Process監視等）
- `models.rs` - 共用データモデル
- `main.rs` - Rust版エントリーポイント

## アーキテクチャ

```
Entry Points (main.rs)
    ↓
Use Cases (process.rs)
    ↓
Infrastructure (api.rs, audio.rs, process.rs, tasks.rs)
    ↓
Domain (domain.rs) ← 依存なし
```

> [!NOTE]
> 現在、一部の機能（文字起こし等）は `use_cases/process.rs` から `uv run python` を通じて Python 側の `cli.py` を呼び出しています。

## ファイル / モジュール一覧

| モジュール | 役割 | 備考 |
|------------|------|------|
| `main.rs` | CLI引数解析・メインループ | 監視モード、手動処理、同期 |
| `use_cases::process` | 録音データの処理フロー | Gemini連携、Pythonスクリプト呼出 |
| `infrastructure::process` | ターゲットプロセスの監視 | `sysinfo` を使用 |
| `infrastructure::audio` | 音声録音制御 | `cpal`, `hound` を使用 |
| `infrastructure::api` | Gemini / Supabase クライアント | `reqwest` を使用 |
| `infrastructure::tasks` | 処理待ちタスクの管理 | `data/tasks.json` |

## 実行方法

### 自動監視モード

```bash
cargo run -- monitor
```

### CLI操作

```bash
# 特定ファイルの処理
cargo run -- process --file data/recordings/audio.wav

# Supabaseへの同期
cargo run -- sync
```

## Legacy (Python)

移行完了まで、以下の Python ファイルが残存・使用されています：
- `cli.py`: 文字起こし (`transcribe`) や要約 (`summarize`) のバックエンドとして利用。
- `app.py`: 旧メインループ。

