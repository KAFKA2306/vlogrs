import logging
import threading
from datetime import datetime
from pathlib import Path

import numpy as np
import sounddevice as sd
import soundfile as sf

from src.infrastructure.settings import settings

logger = logging.getLogger(__name__)


class AudioRecorder:
    def __init__(self):
        self._base_dir = Path(settings.recording_dir).expanduser().resolve()
        self._thread: threading.Thread | None = None
        self._stop_event = threading.Event()
        self._segments: list[str] = []
        self._lock = threading.Lock()

    def start(self) -> str:
        with self._lock:
            if self._segments:
                return self._segments[-1]
            self._base_dir.mkdir(parents=True, exist_ok=True)
            initial_path = str(
                self._base_dir / datetime.now().strftime("%Y%m%d_%H%M%S.wav")
            )
            self._segments = [initial_path]
            self._stop_event.clear()
            self._thread = threading.Thread(target=self._record_loop, daemon=True)
            self._thread.start()
            return initial_path

    def stop(self) -> tuple[str, ...] | None:
        thread = self._thread
        if not thread:
            return tuple(self._segments) if self._segments else None
        self._stop_event.set()
        thread.join()
        with self._lock:
            self._thread = None
            valid = []
            for path in map(Path, self._segments):
                size = path.stat().st_size if path.exists() else 0
                if size > 100:
                    valid.append(str(path))
                elif path.exists():
                    path.unlink()
                    logger.warning("Deleted empty recording: %s (%d bytes)", path, size)
            return tuple(valid) if valid else None

    @property
    def is_recording(self) -> bool:
        return self._thread is not None and self._thread.is_alive()

    def _record_loop(self):
        start_time = last_check = datetime.now()
        last_size = 0

        while not self._stop_event.is_set():
            now = datetime.now()
            if (now - start_time).total_seconds() > 1800:
                new_path = str(
                    self._base_dir / datetime.now().strftime("%Y%m%d_%H%M%S.wav")
                )
                with self._lock:
                    self._segments.append(new_path)
                start_time = now

            current_path = Path(self._segments[-1])
            if current_path.exists() and (now - last_check).seconds > 60:
                size = current_path.stat().st_size
                if size == last_size and size < 1000:
                    logger.warning(
                        "Recording appears empty: %s not growing (size: %d)",
                        current_path,
                        size,
                    )
                last_size, last_check = size, now

            with (
                sf.SoundFile(
                    current_path,
                    mode="w",
                    samplerate=settings.sample_rate,
                    channels=settings.channels,
                    subtype="PCM_16",
                    format="WAV",
                ) as file,
                sd.InputStream(
                    samplerate=settings.sample_rate,
                    channels=settings.channels,
                    blocksize=settings.block_size,
                ) as stream,
            ):
                while (
                    not self._stop_event.is_set()
                    and (datetime.now() - start_time).total_seconds() <= 1800
                ):
                    data, _ = stream.read(settings.block_size)
                    rms_source = (
                        np.frombuffer(data, dtype=np.int16)
                        if isinstance(data, bytes)
                        else data
                    )
                    if rms_source.size > 0:
                        rms = np.sqrt(np.mean(np.square(rms_source)))
                        if rms > settings.silence_threshold:
                            file.write(data)
