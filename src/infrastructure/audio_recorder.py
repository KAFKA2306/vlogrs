import logging
import os
import threading
from datetime import datetime

import numpy as np
import sounddevice as sd
import soundfile as sf

from src.infrastructure.settings import settings

logger = logging.getLogger(__name__)


class AudioRecorder:
    def __init__(self):
        self._base_dir = settings.recording_dir
        self._thread: threading.Thread | None = None
        self._stop_event = threading.Event()
        self._segments: list[str] = []
        self._lock = threading.Lock()

    def start(self) -> str:
        with self._lock:
            if self._segments:
                return self._segments[-1]
            initial_path = os.path.join(
                self._base_dir, datetime.now().strftime("%Y%m%d_%H%M%S.flac")
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
            for path_str in self._segments:
                size = os.path.getsize(path_str) if os.path.exists(path_str) else 0
                if size > 100:
                    valid.append(path_str)
                elif os.path.exists(path_str):
                    os.unlink(path_str)
                    logger.warning(
                        "Deleted empty recording: %s (%d bytes)", path_str, size
                    )
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
                new_path = os.path.join(
                    self._base_dir, datetime.now().strftime("%Y%m%d_%H%M%S.flac")
                )
                with self._lock:
                    self._segments.append(new_path)
                start_time = now

            current_path = self._segments[-1]
            if os.path.exists(current_path) and (now - last_check).seconds > 60:
                size = os.path.getsize(current_path)
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
                    format="FLAC",
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
