---
name: windows-cmd-ps1-ops
description: Windows で VLog の `run.bat`/`bootstrap.ps1`/Rust monitor を運用・障害対応するための実行手順。`UNC パス起動で落ちる`、`cargo が見つからない`、`権限/ExecutionPolicy`、`パス解決`、`強制終了後の再起動`、`ログに原因を残す` といったケースで使用する。
---

# Windows CMD/PS1 Ops (Master Protocol v8.0)

## 1. 役割の完全分離 (Separation of Concerns)
- **Entry Point**: `windows/run.bat` は環境変数の整備（UNC パス解決、パスのパススルー）にのみ専念せよ。ロジックの記述は「重罪」である。
- **Core Controller**: `src/windows/rust/bootstrap.ps1` がビルド、ツールチェーン探索、無限リカバリーループを管理する唯一の真実となる。
- **Execution Body**: `vlog-rs.exe monitor` をマスターバイナリとして運用せよ。

## 2. パスとツールチェーンの正規化
- **絶対パス解決**: `Resolve-Path` を常用し、`\\wsl.localhost` 経由の直接起動を避け、一時的なドライブレター割当（`pushd`）を利用して作業ディレクトリを `C:\` 側へ正規化せよ。
- **Toolchain Discovery**: `where.exe cargo` が失敗する場合、`$env:USERPROFILE\.cargo\bin` を直接探索するフォールバックを `bootstrap.ps1` 内に実装せよ。

## 3. 自律復旧 (Autonomous Resilience)
- **無限再試行**: プロセスが異常終了した場合、5秒の待機後に指数バックオフを用いず、即座に再起動を試行せよ。
- **多重起動禁止**: 起動前に `Get-CimInstance Win32_Process` を用い、既存の `vlog-rs` プロセスを強制終了（`Stop-Process -Force`）せよ。

## 4. ログの透明化とクリーンアップ
- **Truncate on Start**: 起動時に過去の `bootstrap.log` を切り詰め、現在の実行コンテキストのみを保持せよ。
- **固定フォーマット**: ログには必ず `timestamp`, `resolved_path`, `exit_code`, `working_dir` を含めること。

## 5. 受け入れ基準 (DoD)
- `run.bat` 実行後、プロセスが 10秒以上存続すること。
- Windows の `Get-Process` で Discord/VRChat を検知し、WSL 側の `data/recordings` に WAV が出現すること。
- クラッシュ時に `logs/windows-rust-bootstrap.log` にパニック理由が 100% 記録されること。

