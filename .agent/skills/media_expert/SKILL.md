---
name: media_expert
description: Technical guidelines for audio processing and image generation.
---

# Media Expert Skill

## 1. Audio Pipeline
- **Quality**: 48,000Hz, 16-bit (または FLAC 圧縮) を標準とする。
- **Whisper**: `large-v3-turbo` モデルを優先し、VAD (Voice Activity Detection) を有効にして無音区間を効率的に処理する。
- **Storage**: `data/recordings/` 配下のファイル命名規則 `YYYYMMDD_HHMMSS.wav` を厳守する。

## 2. Image Generation
- **Consistency**: 小説の内容に基づき、キャラクターや風景の一貫性を保つプロンプトを生成する。
- **Visual Style**: 幻想的かつ温かみのある、VLogの想い出にふさわしいスタイルを指定する。
- **Resolution**: 出力サイズやアスペクト比を、閲覧デバイス (Web Reader) に最適化する。

## 3. Metadata Management
- Supabase への同期時に、適切なタグ (`#VRChat`, `#Memory`, etc.) とメタデータを付与する。
- 画像と日記の 1:1 対応を保証し、リンク切れを防止する。

## 4. Error Resilience
- API レート制限やネットワークエラー時のリトライ戦略を設計する。
- 生成失敗時にはプレースホルダーや代替処理を適切に配置する。
