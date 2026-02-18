# Use Case: Omniscience (Mobile Life)

**「世界は繋がっている。街の音も、散歩の記憶も、すべてがこの CLI に帰結する」**

Mobile Lifeは、VLogシステムにおける「拡張された感覚（Extended Sensor）」です。
「Absolute Physical Reality」の一環として、PCを離れた時間すらもシームレスに記録し、VLogの中心へ統合します。

## 1. Input Source: The Extended Layer (拡張層)

### 1-1. Integration Architecture (File-Based Sync)
本システムは専用のスマートフォンアプリを持ちません。
既存のクラウドストレージと、Linux側の `File Watcher` のみを接点とします。

- **iOS/Android Setup**:
  - `Voice Memos` (iOS) や `Recorder` (Android) の保存先をクラウド同期フォルダに設定。
  - **Photos**: 写真は `OneDrive` / `Google Photos` / `iCloud` のPC同期機能を利用。

- **Linux Ingest Path (`data/inbox/mobile`)**:
  ```bash
  # rclone mount example
  rclone mount onedrive:VLog/Inbox /home/kafka/vlog/data/inbox/mobile --vfs-cache-mode writes
  ```
  - `vlog` デーモンは `inotify` でこのディレクトリを監視し、`CREATE` イベントが発生した瞬間に取り込みを開始します。

### 1-2. Automatic Normalization
- **Audio**:
  - `m4a`, `mp3`, `aac` など多様なフォーマットを `ffmpeg` で `16kHz Mono WAV` に統一します。
  - **Loudness Normalization**: 屋外録音のレベル差を補正するため、`-23 LUFS` への正規化を適用します。
- **Image**:
  - `HEIC` を `JPG/WebP` に変換し、EXIFデータ（撮影日時、GPS）をメタデータとして抽出します。

## 2. Integration with User Stories (物語への統合)

### Story 1: Immediate Connection (Dashboard)
- **「帰宅した瞬間の同期」**
  - **Ingest Widget**:
    - ダッシュボード右下の "External" セクションがアクティブになり、同期の進捗バーが表示されます。
    - `[Syncing] 20260219_Walk.m4a (45%)`
  - **Notification**:
    - 処理完了後、システム通知（`notify-send` equivalent on TUI）が "Theory of Everything Updated" と告げます。

### Story 4: Omniscience (All-Encompassing)
- **「あらゆる場所からの集約」**
  - **Unified Timeline**:
    - モバイルからのログには `source: mobile` タグが付与されますが、表示上はPCのログと完全に統合されます。
    - 「14:00 PCで作業」→「15:00 散歩（モバイルログ）」→「16:00 再び作業」という一連の流れが、断絶なく描かれます。
  - **Context Linking**:
    - 散歩中に思いついたアイデア（音声メモ）が、帰宅後の作業（PCログ）と「トピック検索」で繋がる体験を提供します。

## 3. Reference Milestone

- **Implementation**: [Milestone 2026-02-19](../milestones/2026-02-19.md) - Phase 2 (File Scanner / Watcher)
- **Dependencies**: `rclone`, `ffmpeg`, `exiftool`

---
**Goal**: PCを離れても、VLogシステムはあなたの人生を記録し続けているという安心感を保証する。
