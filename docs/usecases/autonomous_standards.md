# 自律運用の基準（基本原則）

このドキュメントでは、VLogが「製品」として成立するための完全自律運用の基準を、実装レベルのステートマシンおよび技術定数として定めます。

```mermaid
stateDiagram-v2
    [*] --> Idle
    Idle --> Monitoring: Process Detected
    Monitoring --> Recording: Audio > Gate (2s)
    Recording --> Finalizing: Inactivity > 600s or Process End
    Finalizing --> AIProcessing: Task Created
    AIProcessing --> Complete: Assets Generated
    AIProcessing --> Error: API Error / Crash
    Error --> Retrying: Backoff Applied
    Retrying --> AIProcessing
    Complete --> Idle
```

## 1. 人間の介入ゼロ（Zero-Ops Protocol）
- **自動検知ロジック**: `infrastructure/process.rs` にて `libc` または `sysinfo` を介し、ターゲットプロセス（例: `Discord.exe`）の出現を 1.0s の解像度で監視。
- **自動処理パイプライン**: `use_cases/process.rs` が `TaskRepository` から `pending` を抽出し、`tokio::spawn` によるマルチスレッド並列処理を自律的にスケジューリング。

## 2. 壊れても勝手に直る（Crash-Only Architecture）
- **ステート・チェックポインティング**: 録音中の PCM データは 30 秒ごとに `data/recordings/partial/` にフラッシュ。
- **Auto-Restart ポリシー**: `systemd` ユニット設定における `Restart=always` および `RestartSec=10` の強制適用。
- **リソース隔離**: メインループ（Monitoring）と重い AI 処理（Processing）を別プロセスまたは独立した Async Runtime で分離。

## 3. 自己修復メカニズム（Self-Healing Logic）
- **Jitter 付き指数バックオフ**: `llm.rs` における再試行間隔の算出式：`min(cap, base * 2^n) + random_jitter`。
- **データ不備の自動スキャン**: 起動時に `data/tasks.json` を全スキャンし、`image_path == null` の項目を自動的に再生成キューへ投入。
- **OOM 保護**: `cgroups` (Linux) を用いたプロセスメモリ制限 と、制限接近時の GC 強制発動。

## 4. データの絶対保護（Data Safety Standard）
- **ACID 準拠の保存**: `data/status.json` 等の更新時、`rename(tmp, target)` によるアトミック性を保証。
- **非対称同期**: クラウド側（Remote）の削除操作をローカル（Local）に波及させない「一方向追記型シンク」の採用。
- **ID 衝突回避**: `Snowflake ID` または `UUID v7`（時間順序付き）を用いた全リソースのユニーク性担保。

## 4. 物語の真正性と品質（Narrative Integrity & Quality）
AIが生成する物語の価値を保証するための、厳格な定量的・定性的基準です。

- **Absolute Physical Reality原則**:
    - 出力テキストからVR関連用語（VR, Virtual, HMD, アバター等）を100%排除。`config.yaml` の `prompt_filters` と連動。
- **Curatorによる自動検閲**:
    - `prompts.yaml` の `curator` ロジックを用い、全チャプターを5段階（正規分布）で評価。
    - **分布の定義**:
        - 3 (Standard): 68% (期待通り)
        - 4 (Excellent): 15% (情緒的/独創的)
        - 5 (Masterpiece): 2% (稀有な完成度)
- **Show, Don't Tell 指標**:
    - 感情の直接記述を避け、物理的ディテールや動作による描写が50%以上を占めること。

---

## 網羅的達成基準（Definition of Done）

本システムの「自律稼働」および「物語生成」が達成されたと判断するための具体的指標です。

### 1. 運用・信頼性（Zero-Ops / Crash-Only）
- [ ] **168時間（1週間）無人連続稼働の達成**
    - 期間中、メモリ使用量の増加（RSS）が初期値の +20% 以内に収まっていること。
    - `systemd` によるプロセスの「予期せぬ終了/再起動」が 0 回であること。

### 2. データ・品質（Narrative Excellence）
- [ ] **物語の「標準品質(3)」以上の維持率**: 全生成データの90%以上がスコア3以上であること。
- [ ] **WER（文字起こし誤り率）**: 平均 5.0% 以下を維持していること。
- [ ] **VR用語混入率 0%**: 生成された全ての小説において、禁止用語が一切含まれないこと。

### 3. 可観測性（Observability）
- [ ] **稼働統計の可視化**: `vlog-status` コマンドにより、成功率、評価スコア分布、処理待ち数が即座に確認できること。

---
**総評**: 上記の全項目が「済(x)」となった状態をもって、VLog システムは「製品レベルの自律性」および「文学的価値」を獲得したと認定されます。

