import os
import sys
import time
import wave
import collections
import numpy as np
import pyaudio
import webrtcvad
import shutil
import uuid
import logging
from datetime import datetime

# Milestone 19: 16kHz, 16bit, Mono
FORMAT = pyaudio.paInt16
CHANNELS = 1
RATE = 16000
CHUNK_DURATION_MS = 30  # Milestone 20: 30ms latency
CHUNK_SIZE = int(RATE * CHUNK_DURATION_MS / 1000)

# VAD Parameters
VAD_AGGRESSIVENESS = 3
PADDING_MS = 500  # Milestone 21: 500ms pre-recording buffer
SILENCE_MARGIN_MS = 1000  # Milestone 22: 1s silence margin

# Shared Path
INBOX_DIR = os.getenv("VLOG_INBOX_DIR", "Z:/vlog/inbox/audio")

logging.basicConfig(level=logging.INFO, format="%(asctime)s [%(levelname)s] %(message)s")
logger = logging.getLogger(__name__)

class AudioRecorder:
    def __init__(self):
        self.pa = pyaudio.PyAudio()
        self.vad = webrtcvad.Vad(VAD_AGGRESSIVENESS)
        self.buffer = collections.deque(maxlen=int(PADDING_MS / CHUNK_DURATION_MS))
        self.recording = False
        self.recorded_chunks = []
        self.silent_chunks = 0
        self.max_silent_chunks = int(SILENCE_MARGIN_MS / CHUNK_DURATION_MS)

    def start(self):
        stream = self.pa.open(format=FORMAT, channels=CHANNELS, rate=RATE,
                             input=True, frames_per_buffer=CHUNK_SIZE)
        
        logger.info("VLog Windows Audio Recorder started (High Priority)")
        
        while True:
            try:
                data = stream.read(CHUNK_SIZE, exception_on_overflow=False)
                is_speech = self.vad.is_speech(data, RATE)

                if not self.recording:
                    if is_speech:
                        logger.info("Speech detected. Starting recording...")
                        self.recording = True
                        self.recorded_chunks = list(self.buffer)
                        self.recorded_chunks.append(data)
                        self.silent_chunks = 0
                    else:
                        self.buffer.append(data)
                else:
                    self.recorded_chunks.append(data)
                    if not is_speech:
                        self.silent_chunks += 1
                        if self.silent_chunks > self.max_silent_chunks:
                            logger.info("Silence detected. Stopping recording...")
                            self.save_and_reset()
                    else:
                        self.silent_chunks = 0
            except Exception as e:
                # Milestone 25: Crash-Only design - let it die if critical
                logger.error(f"Critical stream error: {e}")
                sys.exit(1)

    def save_and_reset(self):
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        temp_filename = f"vlog_audio_{timestamp}_{uuid.uuid4().hex[:8]}.wav"
        temp_path = os.path.join(os.getenv("TEMP", "."), temp_filename)

        with wave.open(temp_path, 'wb') as wf:
            wf.setnchannels(CHANNELS)
            wf.setsampwidth(self.pa.get_sample_size(FORMAT))
            wf.setframerate(RATE)
            wf.writeframes(b"".join(self.recorded_chunks))

        # Milestone 24: Atomic move to Shared Folder
        if os.path.exists(INBOX_DIR):
            final_path = os.path.join(INBOX_DIR, temp_filename)
            try:
                shutil.move(temp_path, final_path)
                logger.info(f"Saved: {final_path}")
            except Exception as e:
                logger.error(f"Failed to move file to {INBOX_DIR}: {e}")
        else:
            logger.warning(f"Inbox dir {INBOX_DIR} not found. File kept at: {temp_path}")

        self.recording = False
        self.recorded_chunks = []
        self.silent_chunks = 0
        self.buffer.clear()

if __name__ == "__main__":
    # Milestone 28: High Priority
    try:
        import psutil
        p = psutil.Process(os.getpid())
        p.nice(psutil.HIGH_PRIORITY_CLASS)
    except Exception:
        pass

    recorder = AudioRecorder()
    recorder.start()
