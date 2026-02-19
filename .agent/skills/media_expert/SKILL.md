---
name: media_expert
description: 音声処理と画像生成に関する技術的ガイドライン。
---

# Media Expert Skill

## 1. オーディオパイプライン (物理現実の捕捉)
- **フォーマット標準化**: [src/domain/constants.rs](file:///home/kafka/vlog/src/domain/constants.rs) を参照。
  - **Capture**: ハードウェア・ネイティブ (48kHz/Stereo 推奨)
  - **Processing**: 認識用正規化 (16kHz/Mono)
  - **Opus 圧縮**: 文字起こし完了後の最終保存は `24kbps VBR` の Opus 形式とし、WAV は即座に削除せよ。
- **発話検知 (VAD)**:
  - `webrtcvad` を使用。500ms のプレ録音バッファを保持し、会話の頭欠けを 100% 阻止せよ。

## 2. 永続化と整合性 (SQLite-First)
- **Primary Storage**: 
  - すべてのイベント、メタデータ、資産パスは `SQLite 3` (WALモード) に一元管理される。Supabase は「二次的な同期先」であり、ファイルシステムと SQLite が唯一の真実となる。
- **ディレクトリ規則**:
  - `data/recordings/`: 生音声 (WAV) および圧縮音声 (Opus)
  - `data/transcripts/`: Whisper 出力 (JSON)
  - `data/summaries/`: 日次・個別の要約 (JSON/Markdown)
- **Self-Consistency Engine**:
  - 生成された要約/物語が元のログデータと 99% 以上の整合性を持つか、LLM を用いて自動検証せよ。

## 3. Atmospheric Synthesis (情緒的補完)
- **沈黙の解釈**:
  - 録音や活動ログの空白時間は「欠落」ではなく、その場の「空気感」（平穏、孤独、集中）として解析対象に含めよ。
- **環境音統合**:
  - 特定のアプリ（YouTube等）の視聴履歴を「生活の背景音」として物語の文脈に織り交ぜる。

## 4. 画像生成と視覚的証跡
- **出力仕様**:
  - アスペクト比 `9:16`, `1024x1792`, `.webp` (品質 90)。
  - スタイル: 「温かみのある、哀愁漂う、デジタルと物理の境界が曖昧な質感」。
- **メタデータ紐付け**: 
  - 生成画像は必ず `novels` テーブルの特定の日付/イベントに関連付けられ、物理的証拠として機能しなければならない。

## 5. エラー耐性 (Zero-Fallback)
- **即時停止**: ffmpeg の未検出、モデルロード失敗、DB ロック等の異常時は、代替処理を行わず即座にパニック終了し、クリーンな再起動を待て。
- **ログの絶対性**: 処理の各ステップ（変換、認識、保存）は `tracing` を通じて 1ミリ秒 精度で記録されなければならない。

