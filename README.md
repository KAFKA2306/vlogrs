# VRChat Auto-Diary

VRChatプレイ中の音声を自動録音し、文字起こし後に日記形式で要約するツール。

## セットアップ

```bash
uv sync
cp .env.example .env
# .envにGOOGLE_API_KEYを設定
```

## 使い方

### 自動監視モード

```bash
task up      # サービス起動
task status  # 状態確認
task logs    # ログ確認
task down    # サービス停止
```

### 手動処理

```bash
task record                         # 録音
task transcribe FILE=audio.wav      # 文字起こし
task summarize FILE=transcript.txt  # 要約
task process FILE=audio.wav         # 一括処理
```

## 設定

- `.env`: APIキー (GOOGLE_API_KEY)
- `config.yaml`: プロセス監視、音声、Whisper、Gemini設定

## 構成

```
recordings/   音声ファイル
transcripts/  生トランスクリプト
summaries/    日記形式要約
src/          ソースコード
```
