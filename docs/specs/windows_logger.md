# Windows Logger Specification

本ドキュメントは、VLogシステムのコンポーネントである **Windows Logger (vlog-win-agent)** のアーキテクチャおよび実装詳細を定義します。

## 1. Goal (目的)
「Absolute Physical Reality（絶対的な現実）」をWindowsのデジタル領域まで拡張すること。
Rust製の軽量エージェントを用いて、以下の情報を**不可視かつ自律的**に収集・転送します。

- **Window Focus**: 今、何を見ているか？（コンテキストスイッチの記録）
- **Media Activity**: 今、何を聞いているか？（環境音としてのBGM記録）
- **Idle Status**: 今、席にいるか？（入力活動の有無）

## 2. Technical Stack (技術スタック)

- **Language**: Rust 1.84+ (Stable)
- **Target**: `x86_64-pc-windows-msvc`
- **Dependencies**:
  - `windows` (0.59): Win32 APIへのアクセス (`UI::WindowsAndMessaging`, `Media::Control`, `System::Threading`)
  - `serde`, `serde_json`: イベントデータのシリアライズ
  - `chrono`: ISO 8601 タイムスタンプ (`Local`)
  - `anyhow`: エラーハンドリング（Crash-Only哲学に基づき、回復不能なエラーは即座にパニック・再起動させる）
  - `directories`: 設定ファイル・データ保存先の解決

## 3. Data Architecture (データ構造)

全てのイベントは単一の構造体（Enum）として定義し、JSONL形式で出力します。

```rust
use serde::Serialize;
use chrono::{DateTime, Local};

#[derive(Serialize)]
#[serde(tag = "type", content = "data")]
pub enum LogEvent {
    WindowFocus {
        app_name: String,      // e.g., "Code.exe"
        window_title: String,  // e.g., "windows_logger.md - vlog"
        process_id: u32,
    },
    MediaPlaying {
        app_id: String,        // e.g., "Spotify"
        title: String,
        artist: String,
        status: String,        // "Playing", "Paused"
    },
    SystemIdle {
        duration_sec: u64,     // ユーザー入力がない秒数
    },
    AgentHeartbeat {
        version: String,
        uptime_sec: u64,
    },
}

#[derive(Serialize)]
pub struct LogEntry {
    timestamp: DateTime<Local>,
    #[serde(flatten)]
    event: LogEvent,
}
```

## 4. Signal Processing (信号処理)

### The Eye: Window Polling Loop
- **Polling Rate**: 1000ms (1Hz)
- **API**:
  - `GetForegroundWindow()`: アクティブなHWndを取得。
  - `GetWindowThreadProcessId()`: プロセスIDを取得。
  - `OpenProcess()` + `QueryFullProcessImageNameW()`: 実行ファイル名を取得。
  - `GetWindowTextW()`: ウィンドウタイトルを取得。
- **Filtering**:
  - 直前のウィンドウハンドルと同じ、かつタイトルも変化していない場合はログ出力をスキップする（重複排除）。

### The Ear: Global Media Control
- **Target**: Windows 10/11 `SystemMediaTransportControls` (SMTC)
- **Mechanism**:
  - `GlobalSystemMediaTransportControlsSessionManager::RequestAsync()` でセッションマネージャーを取得。
  - `GetCurrentSession()` でアクティブなセッションを取得。
  - `TryGetMediaPropertiesAsync()` でメタデータを取得。
- **Trigger**:
  - ポーリング（5秒間隔）またはイベントリスナー（`PlaybackInfoChanged`）で変更を検知。

## 5. Transport Layer (物理転送層)

### Strategy: "Shared Folder + Atomic Move"
VLogシステムの基本哲学である「ファイルシステムこそがAPIである」に従います。

1. **Buffer**:
   - ローカル（`%LOCALAPPDATA%\vlog\buffer`）に一時ファイルを作成 (`current.jsonl`)。
   - イベント発生ごとに追記（Append）。

2. **Commit**:
   - 一定時間（例: 10分）または一定サイズ（例: 1MB）でローテーション。
   - ファイルをクローズし、共有フォルダ（`Z:\vlog\inbox\windows`）へ移動。
   - 移動はアトミックに行う（または `copy` -> `delete`）。

3. **Fallback**:
   - 共有フォルダが見つからない場合（オフライン）、ローカルに溜め続ける。
   - 接続回復時に順次転送する。

## 6. Installation & Deployment

### Build
```bash
cargo build --release --target x86_64-pc-windows-msvc
```

### Installation
PowerShellスクリプトによるワンライナーインストール：
1. `vlog-win-agent.exe` を `%LOCALAPPDATA%\vlog\bin` に配置。
2. スタートアップフォルダへのショートカット作成、またはタスクスケジューラへの登録（トリガー: ログオン時）。
3. `config.toml` の生成（出力先パスなどを記述）。

## 7. Crash-Only Considerations
- エージェントはステートレスであるべきです。
- 起動時に「前回正常終了したか」をチェックしません。単に新しいログを書き始めます。
- エラー（権限不足、API失敗）が発生した場合、独自の再試行ロジックを持たず、速やかにパニックしてWindowsのサービス管理機能（もしサービス化する場合）または単純な再起動スクリプトに委ねます。

---
**Privacy Note**:
- **除外リスト**: `config.toml` に `blacklist_apps` (例: `KeePass.exe`, `PrivateBrowsing`) を定義し、該当アプリのウィンドウタイトルは `***`  としてマスクします。
