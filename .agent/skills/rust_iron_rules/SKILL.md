---
name: rust_iron_rules
description: Strict adherence to Rust development standards and project "Iron Rules".
---

# Rust Iron Rules Skill

## 1. Minimalist Code
- 「コードは負債である」と考え、最小限の実装を心がける。
- 未使用のライブラリ、関数、変数は即座に削除する。
- コメントは一切書かない。コードそのもので意図を伝える。

## 2. Clean Architecture
- `domain`, `use_cases`, `infrastructure` の分離を徹底する。
- 依存関係は外側から内側へ (`infra` -> `use_case` -> `domain`) のみ許容する。
- `anyhow` を使用した統一的なエラーハンドリングを行う。

## 3. Strict Quality Control
- 全ての PR またはコミット前に `cargo clippy` と `cargo fmt` を実行する。
- 警告は全て修正し、`-D warnings` をクリアする状態を維持する。
- 1つのファイルが200行を超える場合は、機能的な境界で積極的に分割する。

## 4. Modern Rust Patterns
- `tokio` を使用した効率的な非同期処理を推奨。
- `derive` マクロを適切に使用し、ボイラープレートを削減する。
- 型安全性を最大限活用し、ランタイムエラーを未然に防ぐ。

## 5. Context7 Mastery (Layers 1, 2, 5, 6)
- **Runtime & Environment**: 常に安定したツールチェーンと、不純物のない `.env` を前提とする。
- **Structural Integrity**: 200行の壁を「構造的負債」の臨界点として扱い、積極的に分解する。
- **Logic Purity**: `main.rs` はオーケストレーターに徹し、重厚なロジックは `use_cases` にカプセル化する。
