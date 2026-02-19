---
name: windows-cmd-ps1-ops
description: Windows で VLog の `run.bat`/`bootstrap.ps1`/Rust monitor を運用・障害対応するための実行手順。`UNC パス起動で落ちる`、`cargo が見つからない`、`権限/ExecutionPolicy`、`パス解決`、`強制終了後の再起動`、`ログに原因を残す` といったケースで使用する。
---

# Windows CMD/PS1 Ops (Master Protocol v8.0)

## 目的
Windows 側で VRChat/Discord 検知から録音開始までを安定稼働させる。
以下を必ず満たす。

- エントリポイントを 1 つに固定する: `windows/run.bat` (Thin Wrapper)
- 実行ロジックを 1 つに固定する: `src/windows/rust/bootstrap.ps1` (Core Controller)
- プロトコル v8.0 以降、`run.bat` にロジックを書き込まない。

## 入力として先に集める情報
次の 6 点を最初に取得する。

1. 実行場所: `cmd.exe` でどのパスから起動したか
2. 実行コマンド: `run.bat` をどう起動したか
3. 標準出力の先頭 30 行
4. `logs/windows-rust-bootstrap.log` の末尾 100 行
5. `logs/windows-rust-monitor.log` の末尾 100 行
6. Windows 側 `cargo` の存在 (`where cargo`)

## 具体コンテキスト（このプロジェクトで実際に起きた事象）
次の症状をそのままトリガーとして扱う。

- `UNC パスはサポートされません。Windows ディレクトリを既定で使用します。`
- `用語 'cargo' は ... 認識されません`
- `ファイル名、ディレクトリ名、またはボリューム ラベルの構文が間違っています。`
- `run.bat` 実行後に即終了する
- 強制終了後に「勝手に終了された」状態になる

## 実行ワークフロー

### 1) 起動方式を正規化する
`\\wsl.localhost\...` 直下からの `cmd.exe` 起動を避ける。

- 推奨: Windows ローカルパスへ移動して起動する
- 代替: `pushd \\wsl.localhost\Ubuntu-22.04-LTS\home\kafka\vlog\windows` で一時ドライブ割当後に実行する

`run.bat` は PowerShell を呼ぶ薄いラッパーに保つ。

### 2) 前提ツールを検証する
PowerShell で次を順に実行する。

```powershell
where.exe cargo
where.exe rustup
$env:Path
```

`cargo` が見つからなければ `C:\Users\<User>\.cargo\bin\cargo.exe` を直接解決する。

### 3) 権限と実行ポリシーを検証する
PowerShell で次を確認する。

```powershell
Get-ExecutionPolicy -List
Unblock-File .\windows\run.bat
Unblock-File .\src\windows\rust\bootstrap.ps1
```

`Restricted` 等でブロックされる場合はユーザースコープで許可する。

### 4) パス解決を絶対化する
`bootstrap.ps1` で実施する原則:

- `$MyInvocation.MyCommand.Path` から基準ディレクトリを作る
- `Resolve-Path ... | Select-Object -ExpandProperty ProviderPath` を使う
- `cmd /c` に渡す作業ディレクトリは `C:\` 側へ正規化する

### 5) 停止と再起動を分離する
Linux 側停止 (`pkill`) と Windows 側停止 (`Stop-Process`) を分けて実行する。

Linux 側:

```bash
pkill -f "target/debug/vlog-rs monitor|vlog-rs monitor|windows-rust"
```

Windows 側:

```powershell
Get-CimInstance Win32_Process |
  Where-Object { $_.CommandLine -match 'vlog|windows-rust|bootstrap|run.bat' } |
  ForEach-Object { Stop-Process -Id $_.ProcessId -Force }
```

再起動時は必ず単一起動を守る（多重起動禁止）。

### 6) エラーをログに固定フォーマットで残す
`bootstrap` 側ログ行に最低限次を含める。

- timestamp
- resolved cargo path
- launch command
- working directory
- exit code
- restart delay

## 同一役割ファイルを 1 つにするルール
重複責務を作らない。

- 実行入口: `windows/run.bat` のみ
- 監視制御: `src/windows/rust/bootstrap.ps1` のみ
- 監視本体: Rust monitor バイナリのみ

同じ責務の `.bat` や `.ps1` を増やさない。

## 受け入れ基準
次を全て満たしたら復旧完了と判定する。

1. `run.bat` 実行後 10 秒以上プロセスが存続する
2. `windows-rust-bootstrap.log` にクラッシュ理由が必ず記録される
3. `cargo` 未解決エラーが再発しない
4. UNC 起動でも作業ディレクトリが正規化される
5. Discord または VRChat 起動時に検知ログが出る

## 参照
詳細なエラー別対応は `references/error-map.md` を参照する。
