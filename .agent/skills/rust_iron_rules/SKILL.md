---
name: rust_iron_rules
description: Rust開発基準とプロジェクト「Iron Rules」の厳格な遵守。
---

# Rust Iron Rules Skill

## 1. ミニマリストコード (ゼロ許容度)
- **「コードは負債である」ドクトリン**:
  - **デッドコード**:
    - `cargo udeps` を週次で**必ず**実行すること。
    - `dead_code` リントで検出された未使用の関数、構造体、定数は即座に削除しなければならない。
  - **コメント**:
    - ❌ **絶対禁止**。コードはそれ自体で説明的でなければならない。
    - 例外: `pub` API に対するドキュメンテーションコメント (`///`) は、マクロ生成またはクレート公開に厳密に必要な場合のみ許可される。
  - **複雑性**:
    - **循環的複雑度**: 関数は複雑度スコア < 10 を維持すること (`clippy::cognitive_complexity` で強制)。

## 2. クリーンアーキテクチャ (厳格な階層化)
- **ディレクトリ構造**:
  ```
  src/
  ├── domain/        # 純粋なロジック、外部依存なし
  ├── use_cases/     # アプリケーションのビジネスルール
  ├── infrastructure/# DB, API, ファイルI/O
  └── main.rs        # 依存性の注入とオーケストレーション
  ```
- **依存関係のルール**:
  - `domain` は `use_cases` や `infrastructure` をインポートしてはならない (**MUST NOT** import)。
  - `use_cases` は `infrastructure` をインポートしてはならない (**MUST NOT** import)。
- **可視性**:
  - デフォルトでは `pub(crate)` を使用すること。
  - `pub` は最も外側の境界 (エントリーポイント) に対して**のみ**使用すること。

## 3. 厳格な品質管理
- **必須チェック (Pre-Commit/Pre-Merge)**:
  - **Clippy**: `#![deny(clippy::all, clippy::pedantic, clippy::nursery)]`
    - 許可される例外: `clippy.toml` で正当な理由と共に明示的にホワイトリスト登録されなければならない。
  - **フォーマット**: `cargo fmt -- --check` が成功(**PASS**)すること。
  - **ライセンス**: `cargo deny check licenses` が成功(**PASS**)すること。
- **ファイルサイズ制限**:
  - **ハードリミット**: `200行`。
  - **アクション**: ファイルが200行を超えた場合、即座にサブモジュールへ分割しなければならない。

## 4. モダンRustパターンと安全性
- **エラーハンドリング**:
  - **ライブラリ**: アプリレベルには `anyhow`、ライブラリレベルには `thiserror` を使用。
  - **Unwrap()**: ❌ `src/` 配下での使用は**厳禁** (ただし `#[test]` 内を除く)。`?` 演算子または `expect("REASON")` を使用すること。
- **非同期ランタイム**:
  - `tokio` (最新の安定版) を標準ランタイムとする。
- **型安全性**:
  - **New Type Idiom**: 生の `String` ではなく `struct UserId(String)` などの型付きラッパーを使用すること。
  - **Parse, Don't Validate**: 有効な状態を表す型を使用すること (例: `NonEmptyString`)。

## 5. Context7 Mastery (Layers 1, 2, 5, 6)
- **Layer 1 (Runtime)**:
  - Dockerコンテナは最小フットプリントのために `distroless` または `alpine` を使用すること。
- **Layer 2 (Secret Hardening)**:
  - シークレットは環境変数を介して注入されること。**決して**ハードコードしてはならない。
  - `dotenv` はローカル開発環境でのみ許可される。
- **Layer 5 (Architectural Purity)**:
  - **境界チェック**: 依存関係ルールの違反は**クリティカル**な障害であり、即時のリバート (巻き戻し) を要する。
- **Layer 6 (Logic Safety)**:
  - **ファジング**: クリティカルなパーサーに対しては、`cargo-fuzz` を使用したファジングテストを**必ず**実施すること。
