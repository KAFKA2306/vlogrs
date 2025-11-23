# VRChat Auto-Diary Architecture

## システム概要

VRChatプレイ中の音声を自動録音し、AIで文字起こし・要約して日記化するシステム。

## アーキテクチャ図

```mermaid
graph TB
    subgraph Windows["Windows (Task Scheduler)"]
        TS[Task Scheduler] --> VBS[run_silent.vbs]
        VBS --> BAT[run.bat]
        BAT --> APP[Application]
    end
    
    subgraph Core["Core Loop"]
        APP --> MON[ProcessMonitor]
        MON -->|check_interval| VRC{VRChat実行中?}
        VRC -->|Yes & Not Recording| REC[RecorderService]
        VRC -->|No & Recording| STOP[Stop Recording]
    end
    
    subgraph Processing["Backend Processing"]
        STOP --> PROC[ProcessorService]
        PROC --> TR[Transcriber<br/>Whisper]
        TR --> PRE[Preprocessor<br/>フィラー除去]
        PRE --> SUM[Summarizer<br/>Gemini API]
        SUM --> FILE[summaries/*.txt]
        FILE --> SYNC[sync_supabase]
    end
    
    subgraph Cloud["Supabase"]
        SYNC --> DB[(daily_entries)]
    end
    
    subgraph Frontend["Next.js Frontend"]
        DB -.読取.-> WEB[Vercel<br/>kaflog.vercel.app]
    end
    
    style TS fill:#4CAF50
    style SYNC fill:#FF9800
    style WEB fill:#2196F3
```

## シーケンス図

### VRChat起動時

```mermaid
sequenceDiagram
    participant TS as Task Scheduler
    participant App as Application
    participant Mon as ProcessMonitor
    participant Rec as RecorderService
    participant VRC as VRChat.exe

    TS->>App: 起動 (run.bat)
    loop check_interval秒ごと
        App->>Mon: is_running()
        Mon->>VRC: プロセスチェック
        VRC-->>Mon: 実行中
        Mon-->>App: True
        App->>Rec: start_session()
        Rec-->>App: 録音開始
    end
```

### VRChat終了時

```mermaid
sequenceDiagram
    participant App as Application
    participant Mon as ProcessMonitor
    participant Rec as RecorderService
    participant Proc as ProcessorService
    participant Trans as Transcriber
    participant Sum as Summarizer
    participant Sync as sync_supabase
    participant DB as Supabase

    App->>Mon: is_running()
    Mon-->>App: False
    App->>Rec: stop_session()
    Rec-->>App: RecordingSession
    
    par バックグラウンド処理
        App->>Proc: process_session()
        Proc->>Trans: transcribe_and_save()
        Trans-->>Proc: transcript.txt
        Proc->>Sum: summarize()
        Sum-->>Proc: summary
        Proc->>Proc: summaries/YYYYMMDD_summary.txt保存
        Proc->>Sync: sync_supabase()
        Sync->>DB: upsert(daily_entries)
        DB-->>Sync: OK
    end
```

## 状態遷移図

```mermaid
stateDiagram-v2
    [*] --> Monitoring: Task Scheduler起動
    
    Monitoring --> Recording: VRChat起動検出
    Recording --> Monitoring: VRChat実行中
    Recording --> Processing: VRChat終了検出
    
    Processing --> Transcribing: 録音停止
    Transcribing --> Preprocessing: 文字起こし完了
    Preprocessing --> Summarizing: 前処理完了
    Summarizing --> Syncing: 要約完了
    Syncing --> Monitoring: Supabase同期完了
    
    Monitoring --> Monitoring: チェック継続
```

## コンポーネント構成

```mermaid
graph TB
    subgraph Presentation["Presentation Layer"]
        CLI[cli.py]
        VIEW[view_logs.py]
    end
    
    subgraph Application["Application Layer"]
        APP[app.py<br/>Application]
        REC_SVC[RecorderService]
        PROC_SVC[ProcessorService]
    end
    
    subgraph Domain["Domain Layer"]
        ENT[entities.py<br/>RecordingSession]
    end
    
    subgraph Infrastructure["Infrastructure Layer"]
        MON[ProcessMonitor]
        RECR[AudioRecorder]
        TRANS[Transcriber]
        PREP[Preprocessor]
        SUM[Summarizer]
        SYNC[sync_supabase]
    end
    
    CLI --> PROC_SVC
    APP --> REC_SVC
    APP --> PROC_SVC
    APP --> MON
    REC_SVC --> RECR
    REC_SVC --> ENT
    PROC_SVC --> TRANS
    PROC_SVC --> PREP
    PROC_SVC --> SUM
    PROC_SVC --> SYNC
    PROC_SVC --> ENT
    
    style APP fill:#4CAF50
    style PROC_SVC fill:#FF9800
    style SYNC fill:#2196F3
```

## データフロー

```mermaid
flowchart LR
    VRC[VRChat起動] --> REC[録音<br/>recordings/*.wav]
    REC --> TRANS[文字起こし<br/>transcripts/*.txt]
    TRANS --> CLEAN[前処理<br/>cleaned_*.txt]
    CLEAN --> SUM[要約<br/>summaries/YYYYMMDD_summary.txt]
    SUM --> DB[(Supabase<br/>daily_entries)]
    DB --> WEB[Next.js<br/>Vercel]
    
    style REC fill:#F44336
    style SUM fill:#4CAF50
    style DB fill:#2196F3
```

## 自動化範囲

| コンポーネント | 自動化 | トリガー |
|---|---|---|
| VRChat監視 | ✅ | Task Scheduler起動時 |
| 録音開始/停止 | ✅ | VRChat起動/終了検出 |
| 文字起こし | ✅ | 録音終了時 |
| 前処理 | ✅ | 文字起こし完了時 |
| 要約生成 | ✅ | 前処理完了時 |
| Supabase同期 | ✅ | 要約完了時 |
| Next.js開発 | ❌ | `task reader:dev` |
| Vercelデプロイ | ❌ | `task reader:deploy` |
