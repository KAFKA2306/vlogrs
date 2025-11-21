# VRChat Auto-Diary (vlog)

VRChatのプレイログを自動で記録し、日記としてまとめるツールです。

## 概要

VRChatが起動している間、自動的にマイク音声を録音し、終了後に音声を文字起こしして要約日記を生成します。

## 機能

- **プロセス監視**: VRChatの起動・終了を自動検知
- **自動録音**: プレイ中の音声をバックグラウンドで録音（無音自動カット機能付き）
- **文字起こし**: Faster Whisper（large-v3-turbo）を使用した高速・高精度な文字起こし
- **要約生成**: Gemini APIを使用して日記形式に要約
- **Markdown出力**: 日付ごとにMarkdownファイルとして保存
- **生文字起こし保存**: 文字起こし結果を必ず `transcripts/` に保存
- **CLIツール**: 個別機能を手動実行できるコマンドラインインターフェース

## 必要要件

- Python 3.11+
- Google Gemini APIキー
- （オプション）CUDA対応GPU: Whisper推論を高速化
- （オプション）go-task: タスクランナー

## セットアップ

1. **依存関係のインストール**:

   ```bash
   task setup
   # または
   uv sync
   ```

2. **APIキーの設定（必須）**:

   ```bash
   cp .env.example .env
   # .envを編集してGOOGLE_API_KEYを設定
   ```

3. **設定のカスタマイズ（オプション）**:

   `config.yaml`で以下の設定を調整可能:
   - プロセス監視設定（監視対象、チェック間隔）
   - 音声設定（サンプリングレート、無音しきい値）
   - Whisper設定（モデルサイズ、デバイス、計算型）
   - Gemini設定（使用モデル）
   - ログ保存期間

## 使い方

### クイックガイド

```bash
task
```

すべての利用可能なコマンド一覧が表示されます。

### 自動監視モード（推奨）

#### systemdサービス（Linux）

```bash
# サービスを有効化して起動
task up
# または
systemctl --user enable --now vlog.service

# 状態確認
task status

# ログ確認
task logs

# 再起動
task restart

# 停止・無効化
task down
```

#### Windows

```cmd
.\run.bat
```

#### フォアグラウンド実行（開発時）

```bash
task dev
# または
uv run python -m src.main
```

### 手動録音・処理

#### 録音のみ

```bash
task record
# Ctrl+Cで停止
```

#### 録音ファイルの一括処理

```bash
task process FILE=recordings/20241121_120000.wav
# 文字起こし→要約→日記出力を一括実行
```

#### 個別処理

```bash
# 1. 文字起こし
task transcribe FILE=recordings/20241121_120000.wav

# 2. 要約（テキストファイルから）
task summarize FILE=transcripts/20241121_120000.txt

# 3. 日記出力（要約テキストから）
task write FILE=path/to/summary.txt
```

### CLIツールの直接使用

```bash
# VRChat起動チェック
uv run python -m src.cli check

# 手動録音
uv run python -m src.cli record

# 文字起こし
uv run python -m src.cli transcribe --file recordings/20241121_120000.wav

# 要約
uv run python -m src.cli summarize --file transcripts/20241121_120000.txt

# 日記出力
uv run python -m src.cli write --file path/to/summary.txt

# 一括処理
uv run python -m src.cli process --file recordings/20241121_120000.wav
```

### 開発コマンド

```bash
# コード品質チェック・整形
task lint

# キャッシュクリア
task clean

# Gitコミット
task commit MESSAGE="コミットメッセージ"
```

## ディレクトリ構成

```text
vlog/
├── src/                    # ソースコード
│   ├── main.py            # メインエントリーポイント
│   ├── app.py             # アプリケーション本体
│   ├── cli.py             # CLIツール
│   ├── domain/            # ドメインモデル
│   ├── infrastructure/    # インフラ層（録音、文字起こし、要約等）
│   └── services/          # サービス層
├── recordings/            # 録音データ（自動生成）
├── transcripts/           # 生文字起こしテキスト（自動生成）
├── diaries/               # 日記データ（自動生成）
├── config.yaml            # 設定ファイル（Git管理）
├── .env                   # 環境変数（APIキー等、Git管理外）
├── Taskfile.yaml          # タスク定義
└── vlog.service          # systemdサービス定義
```

## 設定詳細

### config.yaml主要設定

- **process.names**: 監視対象プロセス名（カンマ区切り）
- **audio.sample_rate**: サンプリングレート（デフォルト: 16000Hz）
- **audio.silence_threshold**: 無音カットしきい値（0.01-0.1）
- **whisper.model_size**: Whisperモデル（デフォルト: large-v3-turbo）
- **whisper.device**: 推論デバイス（cpu/cuda/auto）
- **whisper.compute_type**: 計算型（float16推奨、GPU使用時）
- **retention.days**: 古いWAVファイル削除までの日数

詳細は `config.yaml` を参照してください。

## トラブルシューティング

### CUDA関連エラー

- Whisperは自動的にCPUにフォールバックします
- `config.yaml`で `whisper.device: "cpu"` に設定可能

### 録音されない

- VRChatプロセス名が `config.yaml` の `process.names` と一致しているか確認
- `task status` または `uv run python -m src.cli check` でプロセス検出を確認

### ログ確認

- systemd: `task logs`
- ファイル: `vlog.log`
