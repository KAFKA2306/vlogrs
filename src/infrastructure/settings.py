import os
from dataclasses import dataclass
from pathlib import Path

import yaml
from dotenv import load_dotenv

load_dotenv()


@dataclass
class Settings:
    check_interval: int
    recording_dir: str
    transcript_dir: str
    summary_dir: str
    archive_dir: str
    sample_rate: int
    channels: int
    block_size: int
    whisper_model_size: str
    whisper_device: str
    whisper_compute_type: str
    gemini_model: str
    gemini_api_key: str | None
    archive_after_process: bool
    process_names: tuple[str, ...]


def _load_settings() -> Settings:
    config = {}
    if Path("config.yaml").exists():
        with open("config.yaml") as f:
            config = yaml.safe_load(f) or {}

    return Settings(
        check_interval=int(
            os.getenv(
                "VLOG_CHECK_INTERVAL",
                config.get("process", {}).get("check_interval", 30),
            )
        ),
        recording_dir=os.getenv(
            "VLOG_RECORDING_DIR",
            config.get("paths", {}).get("recording_dir", "recordings"),
        ),
        transcript_dir=os.getenv(
            "VLOG_TRANSCRIPT_DIR",
            config.get("paths", {}).get("transcript_dir", "data/transcripts"),
        ),
        summary_dir=os.getenv(
            "VLOG_SUMMARY_DIR",
            config.get("paths", {}).get("summary_dir", "data/summaries"),
        ),
        archive_dir=os.getenv(
            "VLOG_ARCHIVE_DIR",
            config.get("paths", {}).get("archive_dir", "data/archives"),
        ),
        sample_rate=int(
            os.getenv(
                "VLOG_SAMPLE_RATE", config.get("audio", {}).get("sample_rate", 16000)
            )
        ),
        channels=int(
            os.getenv("VLOG_CHANNELS", config.get("audio", {}).get("channels", 1))
        ),
        block_size=int(
            os.getenv(
                "VLOG_BLOCK_SIZE", config.get("audio", {}).get("block_size", 1024)
            )
        ),
        whisper_model_size=os.getenv(
            "VLOG_WHISPER_MODEL_SIZE",
            config.get("whisper", {}).get("model_size", "base"),
        ),
        whisper_device=os.getenv(
            "VLOG_WHISPER_DEVICE", config.get("whisper", {}).get("device", "cpu")
        ),
        whisper_compute_type=os.getenv(
            "VLOG_WHISPER_COMPUTE_TYPE",
            config.get("whisper", {}).get("compute_type", "int8"),
        ),
        gemini_model=os.getenv(
            "VLOG_GEMINI_MODEL",
            config.get("gemini", {}).get("model", "gemini-2.0-flash-exp"),
        ),
        gemini_api_key=os.getenv("VLOG_GEMINI_API_KEY", os.getenv("GOOGLE_API_KEY")),
        archive_after_process=str(
            os.getenv(
                "VLOG_ARCHIVE_AFTER_PROCESS",
                config.get("processing", {}).get("archive_after_process", True),
            )
        ).lower()
        == "true",
        process_names=tuple(
            os.getenv("VLOG_PROCESS_NAMES", "VRChat.exe,VRChat,VRChatClient.exe").split(
                ","
            )
        ),
    )


settings = _load_settings()
