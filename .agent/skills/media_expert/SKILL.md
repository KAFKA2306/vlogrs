---
name: media_expert
description: 音声処理と画像生成に関する技術的ガイドライン。
---

# Media Expert Skill

## 1. オーディオパイプライン (厳格遵守)
- **フォーマット標準化**:
  - 入力音声は処理前に以下のフォーマットへ**必ず**変換すること:
    - **サンプリングレート**: `48000 Hz`
    - **チャンネル**: `1 (Mono)`
    - **コーデック**: `pcm_s16le` (16-bit PCM)
  - **実行コマンド**:
    ```bash
    ffmpeg -i input.wav -ar 48000 -ac 1 -c:a pcm_s16le output.wav
    ```
- **文字起こし (Whisper)**:
  - **モデル**: `large-v3-turbo` **のみ**使用可能。
  - **VAD (発話区間検出)**: **必ず**有効化すること。
    - `min_silence_duration_ms`: `500`
    - `speech_pad_ms`: `400`
  - **出力**: 正確なタイムスタンプ (`start` と `end` 秒) を含むJSON形式。

- **保存規則**:
  - **ディレクトリ**: `data/recordings/` (絶対パス: `/home/kafka/vlog/data/recordings/`)
  - **ファイル名**: `YYYYMMDD_HHMMSS`<`.wav`|`.json`>
    - 例: `20231027_143000.wav`
  - **バックアップ**: 破壊的な処理を行う前に、生の録音データを `data/backup/` へ**必ず**バックアップすること。

## 2. 画像生成 (高忠実度)
- **プロンプトエンジニアリング**:
  - **Positive**: `masterpiece, best quality, ultra-detailed, 8k, cinematic lighting`
  - **Negative**: `worst quality, low quality, bad anatomy, bad hands, text, signature, watermark, username, blurry, artist name`
- **出力仕様**:
  - **アスペクト比**: `9:16` (縦型ストーリーフォーマット)
  - **解像度**: `1024x1792` (または最も効率的な潜在空間サイズ)
  - **フォーマット**: `.webp` (品質: `90`)
- **一貫性チェック**:
  - 生成された画像は、VLogで定義された視覚スタイル（例：「温かみのある、哀愁漂う、VRChat特有のシェーダー感」）と一致しているか検証すること。

## 3. メタデータ管理 (ゼロ許容度)
- **スキーマ検証**:
  - すべてのメタデータは厳密に定義されたJSONスキーマに対して**必ず**検証すること。
  - **必須フィールド**:
    - `id`: UUID v4
    - `timestamp`: ISO 8601 (`YYYY-MM-DDTHH:MM:SSZ`)
    - `tags`: 文字列の配列 (例: `["#VRChat", "#Memory"]`)
    - `assets`: `audio_path` と `image_path` をリンクするオブジェクト
- **同期処理**:
  - Supabaseへの更新は原子的 (Atomic) でなければならない。
  - **リトライポリシー**: ネットワーク障害時は、指数バックオフ (初期値2秒) で最大3回リトライすること。
  - **アセット同期**:
    - **正本 (Source of Truth)**: ファイルシステム (`frontend/reader/public/`) -> データベース (`Supabase`)。
    - **Photos (写真)**:
      - パス: `frontend/reader/public/photos/*.png`
      - テーブル: `novels`
      - マッピング: `filename` (" copy"を除く) -> `date` カラム。
    - **Infographics (インフォグラフィック)**:
      - パス: `frontend/reader/public/infographics/*_summary.png`
      - テーブル: `daily_entries`
      - マッピング: `filename` ("_summary"を除く) -> `date` カラム。
    - **トリガー**: 同期処理は一括画像生成の後、または `task sync:images` 経由で手動実行された際に**必ず**実行すること。

## 4. エラー耐性と回復
- **クラッシュオンリー**:
  - クリティカルなエラー (ffmpeg未検出、モデル読み込み失敗など) が発生した場合、汚染された状態を防ぐためプロセスは即座にパニック/終了 (**MUST panic/exit**) しなければならない。
- **サーキットブレーカー**:
  - 外部API呼び出し (OpenAI, Stable Diffusion) は `30秒` でタイムアウトすること。
