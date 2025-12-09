import os
from pathlib import Path
from typing import TYPE_CHECKING

from src.infrastructure.settings import settings

if TYPE_CHECKING:
    from faster_whisper import WhisperModel


class Transcriber:
    def __init__(self) -> None:
        self._model: "WhisperModel" | None = None

    @property
    def model(self) -> "WhisperModel":
        if self._model is None:
            from faster_whisper import WhisperModel

            self._model = WhisperModel(
                settings.whisper_model_size,
                device=settings.whisper_device,
                compute_type=settings.whisper_compute_type,
            )
        return self._model

    def transcribe(self, audio_path: str) -> str:
        segments, _ = self.model.transcribe(
            audio_path,
            beam_size=5,
            vad_filter=True,
            vad_parameters=dict(min_silence_duration_ms=100, speech_pad_ms=30),
        )
        return " ".join(segment.text.strip() for segment in segments).strip()

    def transcribe_and_save(self, audio_path: str) -> tuple[str, str]:
        base = Path(audio_path).stem
        os.makedirs(settings.transcript_dir, exist_ok=True)
        out_path = Path(settings.transcript_dir) / f"{base}.txt"

        if out_path.exists():
            print(f"Transcript already exists for {base}, skipping Whisper.")
            return out_path.read_text(encoding="utf-8").strip(), str(out_path)

        text = self.transcribe(audio_path)
        out_path.write_text(text + "\n", encoding="utf-8")
        return text, str(out_path)

    def unload(self) -> None:
        self._model = None
