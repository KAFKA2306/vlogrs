---
description: 定期ヘルスチェックとパイプライン保守ワークフロー
---

# Agentic Management Workflow (Destructive Rust Edition)

// turbo-all

## 1. System Health Audit (Native)

### 1.1 Process Integrity
- **systemd Check**: `systemctl --user status vlog --no-pager`。サービスの稼働時間を検証。
- **Log Investigation**: `journalctl --user -n 100 -u vlog --no-pager`。Rust 特有の Panic や Error を監査。
- **Latency Monitoring**: ログ内の処理時間を確認し、Gemini API の遅延が許容範囲内か確認。

### 1.2 Binary Integrity
- **Build Drift**: `cargo check` を実行し、ソースコードとバイナリに乖離がないか確認。

## 2. Processing Audit (Rust-First)

### 2.1 Session Validation
- `data/tasks.json` をロードし、`pending` または `failed` のタスクを抽出。
- `failed` タスクに対しては `cargo run -- process --file [path]` を手動実行し、エラーの原因を徹底究明。

### 2.2 Inventory Matching
- `data/recordings` vs `data/summaries`。
- 1:1 対応が崩れている日付を特定し、不足分を Rust パイプラインで即座に再生成。

## 3. Maintenance (Iron Rules Lockdown)

### 3.1 Totalitarian Quality Check
- **Lint Enforcement**: `task lint` (Rust版) を実行。Clippy の警告を1つも残さない。
- **Format Audit**: `cargo fmt --check`。

### 3.2 Cache & Junk Liquidation
- `find . -name "*.pyc" -o -name "__pycache__" -delete`。**破壊的**に Python の残滓を削除。
- `.ruff_cache` 等の不要なメタデータディレクトリも削除。

## 4. Sync & Verification

### 4.1 Persistence Audit
- `cargo run -- sync` で Supabase との同期を強制。
- 同期完了後、MCP を通じて DB の行数を確認し、ローカルのファイル数との不一致を許さない。

### 4.2 Atomicity
- `task commit MESSAGE="management: destructive maintenance & python purge"`。
