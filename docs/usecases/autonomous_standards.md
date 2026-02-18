# Standards: Zero-Ops & Crash-Only (Autonomous)

**「システムは私が眠っている間も、呼吸するように稼働し続ける」**

このドキュメントでは、VLogが「製品」として成立するための完全自律運用の基準（Autonomous Standards）を定義します。
CLIダッシュボードが「Green Light」を示し続けるための、不可視の憲法です。

## 1. Zero-Ops Protocol (運用ゼロ)

**「管理しないことが、最高の管理である」**

- **Automatic Start (完全自動起動)**
  - `systemd` (Linux) および `impersonated-service` (Windows) により、OS起動と同時に全エージェントが稼働を開始します。
  - **No Manual Intervention**: `task up` すら不要。PCの電源ON = VLog稼働開始。

### 1-1. Systemd Configuration
```ini
# ~/.config/systemd/user/vlog.service
[Unit]
Description=VLog Autonomous Daemon
After=network.target sound.target

[Service]
ExecStart=%h/.cargo/bin/vlog monitor
Restart=always
RestartSec=5s
# Resource Control (OOM Protection)
MemoryMax=512M
CPUQuota=20%

[Install]
WantedBy=default.target
```

- **Self-Healing (自己修復)**
  - ネットワーク切断やAPIエラー（503）に対し、指数バックオフ（Exponential Backoff）で自律的に対処します。
  - **Jitter Implementation**:
    ```rust
    let delay = base_delay * 2u64.pow(retries) + rand::thread_rng().gen_range(0..1000);
    tokio::time::sleep(Duration::from_millis(delay)).await;
    ```

## 2. Crash-Only Architecture (不死性)

**「安全にシャットダウンするな。いつ落ちてもいいように書け」**

### 2-1. State Checkpointing
- **Checkpoint Logic**:
  - メモリ上の状態（`Scanning`, `Processing`）は、変化のたびに `data/status.json` へ即時シリアライズされます。
  - 強制終了時 (`kill -9`) でも、再起動時にこのJSONを読み込むことで、前回の状態から正確に復帰します。

- **Atomic File Operations**:
  - 全てのファイル書き込みは `tempfile -> write -> fsync -> rename` パターンを厳守します。
  ```rust
  let mut temp = NamedTempFile::new_in(&dir)?;
  serde_json::to_writer(&mut temp, &data)?;
  temp.as_file().sync_all()?;
  temp.persist(&target_path)?;
  ```

## 3. Integration with User Stories (物語への統合)

### Story 1: Immediate Connection (Dashboard)
- **「生命維持の鼓動」**
  - ダッシュボードの左上には、システムの健全性を示す "Heartbeat" インジケーターが点滅します。 (`● Green` = Healthy, `● Yellow` = Retrying, `● Red` = Critical)
  - **Error Visibility**:
    - 「エラーが起きた」事実ではなく、「エラーを乗り越えた（Recovered）」回数がポジティブな指標として表示されます。

## 4. Reference Milestone

- **Implementation**: [Milestone 2026-02-19](../milestones/2026-02-19.md) - Phase 2 (Core Features / Log Integration)

---
**Goal**: ユーザーがシステムの存在を忘れ、ただ人生を生きることに集中できる「透明なインフラ」を構築する。
