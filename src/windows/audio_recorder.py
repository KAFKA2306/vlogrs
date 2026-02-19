import datetime
import os
import signal
import sys
import time
import wave

import pyaudio

SAMPLE_RATE = 16000
CHANNELS = 1
CHUNK_DURATION_MS = 30
RECORD_SECONDS = 300
OUTPUT_DIR = "inbox/audio"

def log(msg):
    timestamp = datetime.datetime.now().strftime("%Y-%m-%d %H:%M:%S")
    print(f"[{timestamp}] [AUDIO] {msg}", flush=True)

def ensure_dir(path):
    if not os.path.exists(path):
        os.makedirs(path)

def signal_handler(sig, frame):
    log("Received exit signal. Shutting down...")
    sys.exit(0)

signal.signal(signal.SIGINT, signal_handler)
signal.signal(signal.SIGTERM, signal_handler)

def main():
    ensure_dir(OUTPUT_DIR)
    log("Starting Audio Recorder...")
    
    p = pyaudio.PyAudio()
    info = p.get_host_api_info_by_index(0)
    numdevices = info.get('deviceCount')
    log(f"Found {numdevices} audio devices.")

    stream = p.open(format=pyaudio.paInt16,
                    channels=CHANNELS,
                    rate=SAMPLE_RATE,
                    input=True,
                    frames_per_buffer=int(SAMPLE_RATE * CHUNK_DURATION_MS / 1000))

    log("Audio stream opened. Recording...")

    while True:
        timestamp = datetime.datetime.now().strftime("%Y%m%d_%H%M%S")
        filename = os.path.join(OUTPUT_DIR, f"vlog_audio_{timestamp}.wav")
        
        log(f"Recording to {filename} for {RECORD_SECONDS} seconds...")
        
        wf = wave.open(filename, 'wb')
        wf.setnchannels(CHANNELS)
        wf.setsampwidth(p.get_sample_size(pyaudio.paInt16))
        wf.setframerate(SAMPLE_RATE)

        start_time = time.time()
        while time.time() - start_time < RECORD_SECONDS:
            data = stream.read(
                int(SAMPLE_RATE * CHUNK_DURATION_MS / 1000),
                exception_on_overflow=False,
            )
            wf.writeframes(data)
        
        wf.close()
        log(f"Finished {filename}")

if __name__ == "__main__":
    main()
