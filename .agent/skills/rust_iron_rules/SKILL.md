---
name: rust_iron_rules
description: Rust開発基準とプロジェクト「Iron Rules」の厳格な遵守。
---

# Rust Iron Rules Skill

## 1. 零脂肪（Zero-Fat）プロトコル (超厳格)
- **「コードは負債、説明は敗北」ドクトリン**:
  - **コメント**: ❌ **完全禁止** (`//`, `/*`, `#`)。コードはそれ自体が「記述」でなければならない。
  - **テスト**: ❌ **完全排除**。`tests/` ディレクトリ、`#[test]` マクロ、および単体テストコードを全て削除せよ。
  - **デッドコード**: `cargo udeps` で検出された項目は、将来の可能性を排除して即座に削除する。
  - **複雑性**: 関数は `clippy::cognitive_complexity` < 10 を維持し、1ファイル **200行** を上限とする。超えた場合は即座に破壊・分割せよ。

## 2. 構造的堅牢性 (Crash-Only Design)
- **エラーハンドリング**:
  - **防御的排除**: `match` や `if let` による回復処理 (`Err` 分岐からの復帰) を禁止する。
  - **即時パニック**: 想定外の事態には `.unwrap()` または理由を添えた `.expect()` を用い、5秒以内の自律再起動に全てを委ねる。
  - **言語・ツール**: `anyhow` (App), `thiserror` (Lib) を必須とする。
- **型安全性 (Parse, Don't Validate)**:
  - 生の `String` や `u64` を避け、`UserId(Uuid)` や `Timestamp(i64)` 等の New Type Idiom を徹底する。
  - `Uuid` は時系列順序を保証する **v7** を標準とする。

## 3. アーキテクチャ純粋性 (Totalitarian Layering)
- **ディレクトリ構造**:
  ```
  src/
  ├── domain/        # 純粋なロジック。外部依存は「重罪」。 
  ├── use_cases/     # アプリケーションの進行役。
  ├── infrastructure/# 技術的詳細 (SQLite, API, I/O)。
  └── main.rs        # 依存性注入と死のオーケストレーション。
  ```
- **依存の方向**: `Infrastructure` -> `Use Case` -> `Domain` (DIP)。逆流、および Domain の汚染は即時のリバート対象である。

## 4. 品質と構造化ログ
- **Tracing**: `println!` は「出力汚染」として禁止する。`tracing` クレートによる構造化ログを必須とし、日次ローテーションをバックグラウンドで実施せよ。
- **Clippy**: `#![deny(clippy::all, clippy::pedantic, clippy::nursery, clippy::unwrap_used)]` を基本とするが、Crash-Only 実装のために `unwrap` を意図的に使用する箇所は `expect` で理由を明文化せよ。
- **LTO**: リリースビルド時は `lto = true`, `opt-level = 'z'` を設定し、バイナリの「純粋性」を極限まで高めること。

## 5. Context7 Mastery (Steel Layers)
- **Layer 1 (Runtime)**: `distroless` 等による極小フットプリント。
- **Layer 2 (Secret)**: シークレットのハードコードは万死に値する。OS ネイティブの Credential Manager 等と連携せよ。
- **Layer 5 (Purity)**: 整合性違反は自律修復 (`task repair`) ではなく、根本原因の切除による「根絶」を優先せよ。
- **Layer 6 (Logic)**: 複雑なパーサーには `cargo-fuzz` を必須とする。

