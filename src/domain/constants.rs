pub const APP_DIRS: &[&str] = &[
    "data/recordings",
    "data/transcripts",
    "data/summaries",
    "data/novels",
    "data/photos",
    "data/archives",
    "data/recordings/partial",
    "data/cloud_sync",
    "logs",
    "journals",
    "data/tasks",
];

pub const CONFIG_PATH: &str = "data/config.yaml";
pub const TASKS_PATH: &str = "data/tasks.json";
pub const CLOUD_SYNC_DIR: &str = "data/cloud_sync";
pub const LOGS_DIR: &str = "logs";
pub const LOG_FILE_NAME: &str = "vlog.log";
pub const RECORDINGS_DIR: &str = "data/recordings";

pub const FFMPEG_CMD: &str = "ffmpeg";
pub const PYTHON_CMD: &str = "python";
pub const UV_CMD: &str = "uv";
pub const SQLITE_CMD: &str = "sqlite3";
pub const POWERSHELL_PATH: &str = "/mnt/c/Windows/System32/WindowsPowerShell/v1.0/powershell.exe";

pub const IMAGE_GEN_SCRIPT: &str = "src/scripts/image_gen.py";

pub const STATUS_PENDING: &str = "pending";
pub const STATUS_PROCESSING: &str = "processing";
pub const STATUS_COMPLETED: &str = "completed";
pub const STATUS_FAILED: &str = "failed";

pub const TASK_TYPE_PROCESS_SESSION: &str = "process_session";
pub const TASK_TYPE_SYNC_ACTIVITY: &str = "sync_activity";

pub const SQL_INSERT_EVENT: &str =
    "INSERT INTO life_events (id, timestamp, source_type, metadata) VALUES (?, ?, ?, ?)";
pub const SQL_QUERY_EVENTS: &str = "SELECT id, timestamp, source_type, metadata FROM life_events WHERE timestamp >= ? AND timestamp <= ? ORDER BY timestamp ASC";

pub const SUMMARY_FILE_TEMPLATE: &str = "data/summaries/{}_summary.txt";
pub const NOVEL_FILE_TEMPLATE: &str = "data/novels/{}.md";
pub const PHOTO_FILE_TEMPLATE: &str = "data/photos/{}.png";
pub const EVALUATION_FILE_TEMPLATE: &str = "data/evaluations/{}.json";
pub const CONFIG_DEFAULT_JSON: &str = "[]";
pub const TASKS_FILE_NAME: &str = "tasks.json";

pub const DEFAULT_PROCESS_NAMES: &str = "VRChat,Discord";
pub const DEFAULT_DB_PATH: &str = "/home/kafka/vlog/data/vlog.db";
pub const DEFAULT_SILENCE_THRESHOLD: f64 = 0.02;

pub const BYTES_PER_SECOND_16K_MONO: f64 = 16000.0 * 2.0;

pub const DEFAULT_SAMPLE_RATE: u32 = 48000;
pub const DEFAULT_CHANNELS: u16 = 2;

pub const TARGET_SAMPLE_RATE: u32 = 16000;
pub const TARGET_CHANNELS: u16 = 1;
pub const OPUS_BITRATE: u32 = 12000; // 12kbps Mono for transcription-only

pub const DEFAULT_BITS_PER_SAMPLE: u16 = 16;
pub const WAV_PART_EXTENSION: &str = "wav.part";

pub const AUDIO_LOG_INTERVAL_SECS: u64 = 10;
pub const AUDIO_SLEEP_MS: u64 = 100;
pub const MONITOR_CHECK_INTERVAL_DEFAULT: u64 = 5;
pub const START_DEBOUNCE_SECS_DEFAULT: u64 = 2;
pub const STOP_GRACE_SECS_DEFAULT: u64 = 10;
pub const MIN_RECORDING_SECS_DEFAULT: u64 = 60;
pub const TASK_LOOP_INTERVAL_SECS: u64 = 30;
pub const HEALTH_CHECK_INTERVAL_SECS: u64 = 30;
pub const HEALTH_THRESHOLD_PERCENT: f64 = 90.0;
pub const WATCHER_POLL_INTERVAL_SECS: u64 = 2;

pub const TRANSCRIPT_FILLERS: &[&str] = &[
    "えー",
    "あのー",
    "うーん",
    "えっと",
    "なんて",
    "まあ",
    "そうですね",
    "あー",
    "んー",
    "うん",
    "ふん",
    "あ",
    "はは",
    "ははは",
    "なんか",
    "え",
    "お",
    "ふんふん",
    "ふんふんふん",
    "うんうん",
    "うんうんうん",
    "はいはい",
    "はいはいはい",
    "はいはいはいはい",
    "おー",
    "ああ",
    "んふん",
    "そっか",
    "そっかぁ",
    "そうか",
    "そうなんだ",
    "えへへ",
    "あの",
    "あのね",
    "あのさ",
    "ん",
    "えっと",
];

pub const PROHIBITED_WORDS: &[&str] = &["hmd", "controller", "virtual", "vr"];
