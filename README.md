# VRChat Auto-Diary (vlog)

VRChatのプレイログを自動で記録し、日記としてまとめるツールです。

## 概要
VRChatが起動している間、自動的にマイク音声を録音し、終了後に音声を文字起こしして要約日記を生成します。

## 機能
- **プロセス監視**: VRChatの起動・終了を自動検知
- **自動録音**: プレイ中の音声をバックグラウンドで録音
- **文字起こし**: Faster Whisperを使用した高速・高精度な文字起こし
- **要約生成**: Gemini APIを使用して日記形式に要約
- **Markdown出力**: 日付ごとにMarkdownファイルとして保存

## 必要要件
- Python 3.11以上
- PortAudio (`sudo apt-get install libportaudio2`)
- Google Gemini APIキー

## セットアップ

1. リポジトリをクローン
2. 依存関係をインストール
   ```bash
   uv sync
   ```
3. 環境変数の設定
   ```bash
   cp .env.example .env
   # .envを編集してGOOGLE_API_KEYを設定
   ```

## 使い方

```bash
uv run python -m src.main
```
実行すると常駐し、VRChatの起動を待ち受けます。

## ディレクトリ構成
- `src/`: ソースコード
- `tests/`: テストコード
- `recordings/`: 録音データ（自動生成）
- `diaries/`: 日記データ（自動生成）
