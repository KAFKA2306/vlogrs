---
description: minimize complexity
---

# Agentic Unified Protocol: Iron Rules & Context7 (Totalitarian Edition)

// turbo-all

> [!IMPORTANT]
> **Iron Rules (鉄の掟)**
> この文書は、プロジェクトの品質、整合性、および運用手順を規定する最高位のプロトコルである。
> 全てのエージェントと開発者は、この手順書に従い、例外なく遵守しなければならない。
> 違反は即座にロールバックと修正を要求する。

## 概要と哲学

本プロトコルは、`agentic-optimization.md`（品質と整合性）と `agentic-management.md`（運用と保守）を統合し、`AGENTS.md` のコンテキストを完全に包含した、唯一の真実のソースである。

我々の哲学は **"Rust-First, Quality-Obsessed"** である。
- **Rust**: パフォーマンスと安全性の中核。

- **Data**: 物理的実体（録音ファイル）と論理的実体（要約・小説）の完全な一致。
- **Simplicity**: 「単純明快」こそが正義。複雑さはバグの温床である。重複や古いロジックは即座に廃棄する。
- **Raw Code**: コメント、エラーハンドリング、テストコードは「ノイズ」である。コードそのものが全てを語らなければならない。
- **Architecture**: 精緻なレイヤードアーキテクチャを厳守する。依存の方向は一方向であり、逆流は許されない。

## 目的 (Objective)

本ワークフローの目的は、**「極限の単純化と自律性」**である。
冗長な安全性や説明を排除し、本質的なロジックのみを残すことで、可読性と保守性を最大化する。
同時に、強力な型システムと厳格なアーキテクチャによって、テスト不在でも崩壊しない堅牢性を確保する。

## 完了基準 (Completion Criteria)

本プロトコルが「完了」したとみなされるのは、以下の全条件が満たされた場合のみである。

1.  **Zero Comments**: コード内にコメント（`//`, `#`）が1行も存在しないこと。
2.  **Zero Error Handling**: `try-catch`や`Result`分岐による防御的記述がないこと。想定外は即座にクラッシュ（Panic）させる。
3.  **Zero Tests**: `tests/` ディレクトリや単体テストが存在しないこと。
4.  **Strict Typing**: `Any` 型や型推論任せのコードが存在せず、全ての関数と変数が明示的に型定義されていること。
5.  **Perfect Layering**: `Domain` 層が `Infrastructure` 層に依存するなど、レイヤー違反が一切ないこと。
6.  **Continuous Consolidation**: 似た機能を持つ関数やクラスが統合され、重複がゼロであること。
7.  **Zero Junk**: `tmp`, `temp`, `working` などの一時ディレクトリや、Git管理外の不明なファイルが一切残っていないこと。
8.  **Zero Fragmentation**: 意味もなく分割された小規模ファイルが存在せず、適切にモジュール化されていること。
9.  **Task Reliability**: `Taskfile.yaml` に定義された全てのタスク（`task ...`）が、エラー（終了コード非ゼロ）なく完遂すること。
10. **Zero Fallbacks**: リトライ処理、代替パス、デフォルト値への逃げ道が一切存在しないこと。失敗は失敗として受け入れる。
11. **Fresh Diagrams**: 全ての図解が現状のコードと一致し、かつ簡潔であること。

---

## [Layer 1] Runtime Integrity & Environment Hardening (ランタイムと環境の完全性)

開発・実行環境の健全性は全ての基礎である。"Phantom Bugs"（環境差異による不可解なバグ）を根絶するために、以下の手順を厳守する。

### 1.1 Rust Environment (The Steel Core)
Rust環境は最新かつ安全でなければならない。

1.  **Version Check**:
    - コマンド: `rustc --version`
    - **基準**: バージョン `1.90.0` 以上であること。これ未満の場合は即座に `rustup update` を実行せよ。
2.  **Cargo Health**:
    - コマンド: `cargo check --locked`
    - **基準**: 依存関係のロックファイル (`Cargo.lock`) との不整合がないこと。エラーゼロが絶対条件である。



### 1.3 System Service Status (The Heartbeat)
Backbone services must be immortal.

1.  **Status Check**:
    - Command: `systemctl --user status vlog --no-pager`
    - **Criteria**: `Active: active (running)` must be true for `vlog.service` and `vlog-daily.timer`.
    - **Repair**: If inactive, run `task up` immediately.
2.  **Log Analysis**:
    - コマンド: `journalctl --user -n 100 -u vlog --no-pager`
    - **基準**: "Panic" や "Error" の文字列が含まれていないこと。Rust特有のパニックログは重大なインシデントとして扱う。

### 1.4 Secret Hardening (The Vault)
機密情報は厳重に管理される。

1.  **Entropy Audit**:
    - **検証**: `.env` ファイル内の `GOOGLE_API_KEY` および `SUPABASE_KEY` が有効な形式（`sk-` や `AIza` で始まるハッシュ文字列）であることを確認する。
    - **警告**: プレースホルダー（例: `your_api_key`）の残留は許されない。
    - **検証**: `src/infrastructure/settings.rs` (Rust) が、`.env` の全変数を正しくロードしているか確認する。定義漏れは "Configuration Drift" である。
3.  **Web Env Sync**:
    - **Command**: `task web:env`
    - **Criteria**: `frontend/reader/.env.local` must be perfectly synchronized with root `.env`. No manual copying allowed.

---

## [Layer 2] Data Inventory & Logical Consistency (データの物理的・論理的整合性)

データはプロジェクトの血液である。ファイルの欠落や不整合は「貧血」を意味する。

### 2.1 The Inventory Matrix (物理監査)
物理ファイル（録音）と論理ファイル（要約・小説）の対応関係を監査する。

1.  **Recording vs Summary**:
    - **基準**: `data/recordings/` 内の有効な録音ファイル（WAV/FLAC, 60秒以上）は、必ず `data/summaries/` に対応する要約ファイルを持つ必要がある。
    - **自動修復**: 欠落がある場合は `task process FILE=[path]` を実行して即座に生成する。
2.  **Summary vs Novel**:
    - **基準**: `task process:daily` によって生成された日次要約は、必ず `data/novels/` 内のMarkdown小説として昇華されていなければならない。これが "Pipe Blockage"（パイプ詰まり）の主要因となる。

### 2.2 Ghost Purge (不要データの粛清)
価値のないデータはディスクを汚染する。無慈悲に削除せよ。

1.  **Zero-Byte Files**:
    - コマンド: `find data/ -type f -size 0 -delete`
    - **説明**: 生成に失敗した0バイトのファイルは、次のプロセスの障害となるため削除する。
2.  **Stalled Recordings**:
    - コマンド: `find data/recordings -name "*.tmp" -mmin +60 -delete`
    - **説明**: 60分以上更新のない `.tmp` ファイルは、クラッシュした録音プロセスの残骸である。

### 2.3 Supabase Synchronization (クラウド同期)
ローカルの真実はクラウドの真実と一致しなければならない。

1.  **Force Sync**:
    - コマンド: `task sync` (Rust実装)
    - **基準**: エラーなく完了すること。
2.  **Row Count Audit**:
    - **検証**: `serena` MCPサーバーなどを通じてDBの行数を確認し、ローカルの `find . | wc -l` の結果と整合しているか確認する。乖離は "Sync Drift" であり、許容されない。

### 2.4 Directory Whitelist (ディレクトリホワイトリスト)
未知のディレクトリは許されない。

1.  **Strict Structure**:
    - **許可リスト**: `src`, `data`, `docs`, `frontend`, `.agent`, `.git`, `.cargo` (if local), `target` (Rust build)。これら以外は異物である。
    - **アクション**: 上記以外のディレクトリ（例: `temp`, `old`, `backup`）がルートに見つかった場合、即座に `rm -rf` で削除する。質問は不要である。

---

## [Layer 3] Content Fidelity & Narrative Assurance (コンテンツの品質と文脈)

我々が生成するのは単なるデータではなく、「物語」である。品質への妥協は許されない。

### 3.1 Audio Pipeline Audit (聴覚品質)
`AGENTS.md` に定義された基準を満たしているか確認する。

1.  **Spec check**:
    - **Sample Rate**: 48000 Hz であること。
    - **Channels**: 2 (Stereo) であること。
    - **Silence Threshold**: -40 dB 設定が守られているか、設定ファイル (`config.yaml`) を確認する。
2.  **Duration Check**:
    - **検証**: 60秒未満の音声ファイルは「ノイズ」とみなし、警告を発する（または `config.yaml` の `min_duration` 設定に従い無視する）。

### 3.2 Novel & Narrative Quality (物語品質)
AIが生成した小説の品質を監査する。

1.  **Length Audit**:
    - コマンド: `find data/novels -name "*.md" -size -500c`
    - **基準**: 500文字未満の章は「内容が薄い」と判断し、要確認フラグを立てる。これらは再生成の対象となる。
2.  **The Curator Evaluation**:
    - コマンド: `task curator:eval date=[YYYYMMDD]`
    - **基準**: 信頼性スコア (Reliability Score) が **0.8** 未満の場合、人間によるレビューを必須とする。

### 3.3 Visual Integrity (視覚品質)
小説には挿絵が必要である。

    - **Command**: `task photos:fill`
    - **Description**: Automatically detects dates with missing photos and registers generation tasks.
    - **Quality**: Generated images (`data/photos/*.png`) must be valid. Verify with `identify` or similar tools.

### 3.4 The Repair Agent (Self-Healing)
System must possess self-repair capabilities.

1.  **Execution**:
    - **Command**: `task repair`
    - **Scope**: Fixes broken pipeline states, clears stale locks, and resolves simple inconsistencies. Run this before escalating to manual intervention.

---

## [Layer 4] Architectural Purity & Code Health (アーキテクチャとコードの健全性)

コードは生物である。骨格（アーキテクチャ）が歪めば死ぬ。

### 4.1 Precise Layered Architecture (精緻なレイヤードアーキテクチャ)
依存関係の方向性を絶対とする。

1.  **Layer Definitions**:
    - **Domain**: 純粋なビジネスロジック。外部依存（DB, API, IO）を一切持たない。
    - **Use Case**: アプリケーションの進行役。DomainとInfrastructureを繋ぐ。
    - **Infrastructure**: 技術的詳細（DB, FileSystem, External API）。
    - **Interface (CLI/Web)**: エントリーポイント。
2.  **Dependency Rule**:
    - **Interface** -> **Use Case** -> **Domain** <- **Infrastructure** (DIP: 依存性逆転の原則)
    - **禁止**: `Domain` が `Infrastructure` を import することは重罪である。
    - **監査**: `grep` 等を用いて、上位レイヤーが下位レイヤーを知っていないか定期的に確認する。

### 4.2 Explicit Typing (明示的な型定義)
型はドキュメントであり、契約である。

1.  **No Type Inference**:
    - **Rust**: 関数の引数、戻り値はもちろん、複雑な変数定義でも可能な限り型を明示する。

    - **理由**: テストとコメントがない世界において、型定義のみが唯一の手がかりであるため。

### 4.3 Continuous Consolidation (絶え間なき統廃合)
コードは放置すると増殖する。常に刈り込む。

1.  **Merge & Purge**:
    - **基準**: 「似ているが微妙に違う」コードを見逃さない。共通化（Genarics/Trait/BaseClass）によって一つに統合する。
    - **Micro-files**: 数行しかないファイルや、単一の関数しか持たない「小規模ファイル」は、関連するモジュールに統合する。ファイルの断片化は悪である。
    - **アクション**: 新機能を追加する前に、既存のコードを減らす方法を考える。コードが減る変更は常に善である。

### 4.4 Dynamic Structuring (動的な構造改革)
ディレクトリ構造は固定された聖域ではない。現状に合わなくなった構造は、即座に破壊・再構築せよ。

1.  **Fluidity Rule**:
    - **基準**: 「今のコードベースにとって最適か？」だけが判断基準である。昨日の正解は今日の負債かもしれない。
    - **アクション**: ファイル移動 (`mv`) やディレクトリのリネームを躊躇してはならない。IDEのリファクタリング機能を信頼し、大胆に配置を変えること。構造を変えることにコストを感じてはならない。

### 4.5 The 200-Line Ceiling (複雑性の排除)
1つのファイルが大きすぎることは、責務の分離ができていない証拠である。

1.  **Line Count Audit**:
    - コマンド: `find src -name "*.rs" | xargs wc -l`
    - **基準**: 200行を超えるファイルは "Refactoring Debt"（リファクタリング負債）としてリストアップし、優先的に分割する。



### 4.8 Absolute Necessity (絶対必要性)
存在意義の説明できないファイルは、悪である。

1.  **The "Why" Test**:
    - **基準**: プロジェクト内の全てのファイルについて、「なぜこれが必要か？」「これがなくなると何が困るか？」を即答できなければならない。
    - **アクション**: 「将来使うかも」「念のため」「バックアップ」といって残されているファイルは、例外なく即座に削除する。1つたりとも残してはならない。

### 4.9 Constructive Robustness (構造的堅牢性)
テストがない世界では、コンパイルが最後の砦である。

1.  **Make Invalid States Unrepresentable**:
    - **原理**: 「不正な状態」をコード上で表現不可能にする。
    - **実践**: `Option<T>` を使うべき場所で `null` (またはそれに準ずるもの) を使わない。状態遷移は `Enum` で厳密に管理する。文字列(`String`)ではなく、意味のある型(`struct UserId(String)`)を使う。
    - **ゴール**: 「コンパイルが通れば、それは正しく動く」状態を目指す。

---

## [Layer 5] Raw Logic & Fail Fast (生のロジックと即時停止)

防御的なコードはバグを隠蔽する。

### 5.1 No Error Handling & No Fallbacks (エラーハンドリングとフォールバックの排除)
エラーをハンドリングしてはならない。失敗を隠蔽してはならない。

1.  **Unwrap Everything**:
    - **推奨**: Rustにおける `Result` 型は全て `.unwrap()` する。
    - **禁止**: `match` や `if let Err` によるエラー回復処理を禁止する。
    - **Zero Fallback**: 「APIがダメならキャッシュを使う」「失敗したらデフォルト値を返す」といったフォールバック処理は厳禁である。システムは不整合な状態で稼働し続けるよりも、即座に死ぬべきである（Crash-Only Software）。

### 5.2 No Testing (テストの排除)
テストコードはメンテナンスコストである。

1.  **Test Deletion**:
    - **アクション**: `tests/` ディレクトリ、`*_test.rs` を全て削除する。
    - **代替**: 実際の動作（`task process` 等）をもって検証とする。動けば正義である。

### 5.3 Task Validation (タスクの健全性)
非同期タスクシステムの健全性を確認する。

1.  **Failed Task Hunt**:
    - **検証**: `data/tasks.json` を検査し、status が `failed` になっているタスクを特定する。
    - **処置**: 失敗したタスクは、エラーログ（Panicログ）を直視し、コードを修正して再実行する。

### 5.4 Deep Simplicity (深淵なる単純性)
複雑さは「賢さ」ではなく「愚かさ」の証明である。

1.  **Occam's Logic**:
    - **基準**: 同じ機能を実現するために、より単純な方法が存在する場合、現在のコードは「誤り」である。
    - **アクション**: ライブラリの魔術的機能や、難解なアルゴリズムを避け、誰もが読める「素朴なコード」を書く。for文で書けるなら、複雑なイテレータチェーンを使うな。

---

## [Layer 6] Knowledge & Vision Sync (知識とエージェントの同期)

ドキュメントとコードの乖離は、混乱の元凶である。

### 6.1 Manifest Alignment
エージェントへの指示書 (`AGENTS.md`) と実際の実装 (`src/main.rs`) は同期しているか？

1.  **Verification**:
    - `AGENTS.md` に記載されている `VRChat Monitoring` の仕様（例: 5秒間隔チェック）が、`src/process/monitor.rs` 等の実装と一致しているか確認する。
    - 使用するGeminiモデル (`gemini-3-flash`) が、コード内の定数定義と一致しているか確認する。

### 6.2 Visual Precision (最新かつ簡潔な図解)
図解はコードの鏡である。曇りや歪みがあってはならない。

1.  **Sync or Delete**:
    - **基準**: `docs/diagrams/` のMermaidファイルは、現在のコードと完全に一致していなければならない。少しでも古い図は、誤解を生む「嘘の地図」であるため、更新するか削除する。
2.  **Conciseness**:
    - **基準**: 巨大で複雑な図を書いてはならない。1つの図は1つの概念のみを説明する（Single Responsibility Principle for Diagrams）。詳細すぎる図はメンテナンス不能になるため、抽象度を保つこと。

---

## [Layer 7] Operational Execution (運用実行)

日々の運用は、以下のコマンド群によって遂行される。これらは `Taskfile.yaml` に定義された標準手順である。

### 7.1 Task Integrity (タスクの完全性)
`Taskfile.yaml` は唯一の実行可能なドキュメントである。

    - **Action**: Periodically verify all commands via `task --list` (dry run if possible). Tasks that fail to execute must be fixed or removed immediately. Dead code in `Taskfile.yaml` is toxic.

### 7.2 Daily Routine (The Chain of Life)
The daily heartbeat of the system.

1.  **Status Check**: `task service:status`
2.  **Processing Chain**: `task process:daily`
    - **Flow**: `process:yesterday` -> `process:today` -> `summarize` -> `novel:build` -> `process:pending` -> `curator:eval`
    - **Constraint**: This entire chain must succeed without manual intervention.
3.  **Sync**: `task sync:full` (Force sync all data)

### 7.3 Web UI Operations (The Face)
The frontend must be deployment-ready at all times.

1.  **Development**: `task web:dev` (Localhost:3000)
2.  **Build**: `task web:build` (Must pass without lint/type errors)
3.  **Production**: `task web:start` (Localhost:4000)
4.  **Deployment**: `task web:deploy` (Vercel)

### 7.4 Serena MCP Operations (The Brain Extension)
Management of the Model Context Protocol server.

1.  **Initialization**: `task serena:onboarding` (Index creation)
2.  **Runtime**: `task serena:start` (MCP Server start)

### 7.5 Maintenance (The Hygiene)
- **Weekly**: `task lint` (Rust + Python), `task clean` (Cache purge)
- **Monthly**: `task curator:eval` (Long-term trust analysis)

### 7.6 Emergency (The Red Button)
- **Service Stop**: `task down`
- **Full Reset**: `task clean` -> `task up`
- **Pipeline Repair**: `task repair`

---
*Generated by Antigravity Agent for Project VLOG. Adhere to these protocols or face the compiler's wrath.*
