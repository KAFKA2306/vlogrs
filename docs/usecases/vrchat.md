# Use Case: Deep Immersion (VRChat)

**「私は、その場の空気を、風を、温度を、デジタルな記録としてではなく、物理的な記憶として取り戻したい」**

VLogシステムにおける VRChat は、単なるゲームログではなく、**「別の座標にある物理現実」** として扱われます。
HMDやコントローラーといったインターフェース（ノイズ）を完全に透過し、純粋な体験のみを抽出してシステムの「Time Travel」機能と「Reader」機能に供給します。

## 1. Input Source: The Physical Layer (物理層)

### 1-1. Audio Capture (Environmental)
- **Signal Flow**:
  1. `VRChat` output -> `VBCABLE_Input` (Windows) -> `Stereo Mix` -> `vlog-win-agent`.
  2. `vlog-win-agent` は 48kHz/24bit の高解像度で環境音をキャプチャします。
- **Spatial Audio**:
  - ステレオ位相（Phase）を保持し、後で聴き返したときに「右後ろから声がした」感覚を再現します。
- **Silence Detection**:
  - ローディング画面やAFK（離席）中の無音・定常ノイズを `ffmpeg silencedetect` で検知し、自動カットします。
  ```bash
  ffmpeg -i input.wav -af silencedetect=noise=-50dB:d=5 -f null -
  ```

### 1-2. Visual Context Integration
- **OSC Query / Log Parsing**:
  - `OSC` (Open Sound Control) をリッスンし、アバターパラメータの変更やエモート発火をイベントとして記録します。
  - `output_log.txt` をテールし、ワールド移動（`[RoomManager] Entering Room: ...`）を検知します。
- **Image Generation Prompt**:
  - ワールド名 + 現在時刻 + `Z-Image-Turbo` の美的フィルタ = 「記憶の絵葉書」。
  - 例: `location: "Organism", time: "Night", mood: "Cyberpunk", style: "Cinematic Lighting"`

## 2. Integration with User Stories (物語への統合)

### Story 2: Time Travel (Timeline)
- **「あの時、誰といたか？」**
  - **World Heatmap**:
    - CLIのタイムライン上では、滞在したワールドが色分けされて表示されます（例：青=Chill, 赤=Club, 緑=Nature）。
  - **Friend Log**:
    - `OnPlayerJoined` イベントから「誰と一緒だったか」を抽出し、検索インデックス化します。
    - `/search friend:Menta` で、その友人と過ごした夜だけをフィルタリング可能です。

### Story 3: Deep Immersion (Reader)
- **「VRという言葉を使わずに、その光景を描写する」**
  - **Narrative Transformation**:
    | Raw Log | Forbidden | Narrative (Allowed) |
    | :--- | :--- | :--- |
    | `User logged in` | Logged in | 目覚めると、そこは... |
    | `Joined World: Beach` | World | 潮騒の音が聞こえる浜辺に降り立った。 |
    | `Avatar changed` | Avatar | 鏡の中の自分は、いつもより少し背が高く見えた。 |
  - **Immersive Read**:
    - `vlog-tui` の Reader モードでは、テキストの背景にうっすらと当時のワールドの環境音（波の音など）がループ再生されます。

## 3. System Behavior (自律動作)

- **Auto-Wake via Process Watcher**:
  - `vlog` は `VRChat.exe` (Windows) の起動を `vlog-win-agent` 経由で検知します。
  - 検知後、Linux側の録音・集計タスクが「スタンバイ」状態から「アクティブ」状態へ遷移します（Latency < 5s）。
- **Crash-Only Persistence**:
  - VRChatが予期せずクラッシュした場合、`vlog` は "Blackout" イベントとして記録し、録音を正常に終了させます。
  - 再起動後、"Waking up again" として新しいチャプターが始まります。

## 4. Reference Milestone

- **Implementation**: [Milestone 2026-02-19](../milestones/2026-02-19.md)
  - Phase 3 (Image Support): Sixelによる情景画像の表示。
  - Phase 4 (Integration): `vlog-win-agent` からのOSC/ログデータ受信。

---
**Goal**: VRChatをプレイしているのではなく、「別の現実を生きている」ことをシステムレベルで保証する。
