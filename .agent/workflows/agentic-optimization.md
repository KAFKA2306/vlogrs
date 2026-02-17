---
description: コンテンツ品質改善とコードベース最適化ワークフロー
---

# Agentic Optimization Workflow (Context7 Elite Edition)

// turbo-all

## 1. Context7: Environment & Toolchain Integrity (Layer 1-2)

### 1.1 Fundamental Foundation (Layer 1: Runtime)
- **Rust Toolchain**: `rustc --version` (stable)。環境の不整合を排除。
- **Lockfile Enforcement**: `cargo check --locked`。外部依存の「不純物」を排除。

### 1.2 Configuration Context (Layer 2: Environment)
- **Zero-Drift .env**: `.env.example` との完全同期を検証。
  ```bash
  comm -23 <(grep -Po '^[^#=]+' .env.example | sort) <(grep -Po '^[^#=]+' .env | sort)
  ```

## 2. Context7: Resource & Content Audit (Layer 3-4)

### 2.1 State Context (Layer 3: Current Assets)
- **Storage Profile**: `du -sh data/*`。
- **Inventory Matrix**: Recordings vs Summaries の 1:1 対応を物理的に監査。

### 2.2 Fidelity Context (Layer 4: Data Quality)
- **Empty Archive Purge**: `find data/ -type f -size 0 -delete`。
- **Content Health**: `task curator:eval` によるスコアリングと無音ファイルの抽出。

## 3. Context7: Codebase & Complexity Audit (Layer 5-6)

### 3.1 Structural Context (Layer 5: Architecture)
- **Iron Rule Enforcement**: `cargo clippy -- -D warnings`。警告は負債である。
- **Modular Purity**: 200行超えのファイルを「構造的汚染」とみなし、即座に分解を計画。

### 3.2 Logic Context (Layer 6: Use Cases)
- **Delegation Review**: `main.rs` が薄く保たれ、`use_cases/` にビジネスロジックがカプセル化されているかコードレビュー。
- **Atomic Operations**: 関数が単一責任（SR）を全うしているか検証。

## 4. Context7: Vision & Synchronization (Layer 7)

### 4.1 Knowledge Context (Layer 7: Documentation)
- **Truth Sync**: `README.md` / `AGENTS.md` と `main.rs` の完全な同期。
- **Mermaid Reality**: ダイアグラムが現状の Rust シーケンスを正確に反映しているか監査。

### 4.2 Cloud Synthesis
- **Rust Atomic Sync**: `cargo run -- sync`。
- **Final Consensus**: `task commit MESSAGE="optimization: Context7 alignment & destructive cleanup"`。
