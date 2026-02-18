# Use Case: Immediate Connection (Discord)

**「私は、息をするように他者と繋がり、その鼓動をシステムの一部として感じたい」**

Discordは、VLogシステムにおける「社会的な脳（Social Brain）」として機能します。
CLIダッシュボード上のステータスとして、あるいはタイムライン上の重要なイベントとして、会話という「行為」を物理的に記録・統合します。

## 1. Input Source: The Social Layer (社会層)

### 1-1. Process Architecture
Linuxにおける音声キャプチャは、Pulseaudio/PipeWireのモニターソースを利用します。

- **Monitor Target**:
  - `Discord` プロセスの音声出力を特定し、`pactl` または `pw-link` でキャプチャストリームに接続します。
  - **No-Ops**: 「録画ボタンを押す」という概念は存在しません。VAD (Voice Activity Detection) が音声エネルギー（-40dB thresholds）を検知した瞬間、`ffmpeg` プロセスがバックグラウンドで起動します。

### 1-2. Audio Format Specification
- **Codec**: Opus (128kbps) または FLAC (Level 5)
  - 聴き返すための「音楽」ではなく、解析するための「データ」として最適なフォーマットを選択。
- **Channels**:
  - **Environment (CH1)**: DiscordのSystem Audio（相手の声）。
  - **Self (CH2)**: マイク入力（自分の声）。
  - 自動ミキシングにより、会話の被りを明確に分離した状態で保存します。

```rust
// pseudo-code for VAD trigger
if current_volume > -40.0 && last_active < 300.0 {
    start_recording();
} else if silence_duration > 10.0 {
    stop_recording();
}
```

## 2. Integration with User Stories (物語への統合)

### Story 1: Immediate Connection (Dashboard)
- **「今、この瞬間の繋がり」**
  - ダッシュボードの "Status" ウィジェット (`src/tui/components/dashboard.rs`)：
    ```text
    [Status]
    Discord: ■ Connected (VC: "General")
    Audio  :  ▂▃▅▆▇ (Level Meter)
    Rec    : ● [00:15:32]
    ```
  - `Green Lamp` は、単にプロセスがあるだけでなく、音声パケットが流れていることを示します。

### Story 2: Time Travel (Timeline)
- **「あの夜、何を話したか？」**
  - **Event Indexing**:
    - 会話終了後、`whisper-large-v3` が文字起こしを実行。
    - 生成された JSONL には `speakers: ["User", "FriendA"]` といったメタデータが付与されます。
  - **Visual Search**:
    - タイムライン上では、会話ブロックが波形（Audiowaveform）としてレンダリングされます。
    - `/search "project delta"` とタイプすると、その単語が発せられた瞬間の波形位置へジャンプします。

### Story 3: Deep Immersion (Reader)
- **「脚本（Script）としての会話」**
  - 小説モード（Reader）では、会話ログが以下のように整形されます：
    > **[23:15]** ジンの氷が鳴る音がした。
    >
    > 「それで、新しいアーキテクチャはどうなったの？」彼が尋ねる。
    >
    > 「悪くないよ」私は答えた。「ただ、少し美学に欠ける」
  - **Contextual Imagery**:
    - `Z-Image-Turbo` は会話の感情分析（Sentiment Analysis）を行い、"Chill", "Heated", "Melancholic" といったトーンに合わせて背景色の濃度や照明効果を変化させます。

## 3. System Configuration (config.yaml)

```yaml
discord:
  monitor_device: "alsa_output.pci-0000_00_1f.3.analog-stereo.monitor"
  mic_device: "alsa_input.usb-Blue_Microphones_Yeti_Stereo_Microphone-00.analog-stereo"
  vad_threshold: -40.0 # dB
  silence_timeout: 10 # seconds
  output_format: "flac"
  mix_strategy: "stereo_split" # Left: Remote, Right: Mic
```

## 4. Reference Milestone

- **Implementation**: [Milestone 2026-02-19](../milestones/2026-02-19.md) - Phase 2 (Core Features)
  - `Log Integration` でイベント検知を実装。
  - `Search Engine` で会話内容の検索を実装。

---
**Goal**: 会話を「データ」としてではなく、「人生のサウンドトラック」としてアーカイブする。
