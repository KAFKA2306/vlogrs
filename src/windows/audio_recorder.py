import sys
import time
import os
import datetime
import traceback
import signal

# --- Configuration ---
SAMPLE_RATE = 16000
CHANNELS = 1
CHUNK_DURATION_MS = 30
RECORD_SECONDS = 300 # New file every 5 minutes
OUTPUT_DIR = "inbox/audio"

def log(msg):
    timestamp = datetime.datetime.now().strftime("%Y-%m-%d %H:%M:%S")
    print(f"[{timestamp}] [AUDIO] {msg}", flush=True)

def ensure_dir(path):
    if not os.path.exists(path):
        os.makedirs(path)

# Graceful exit handler
def signal_handler(sig, frame):
    log("Received exit signal. Shutting down...")
    sys.exit(0)

signal.signal(signal.SIGINT, signal_handler)
signal.signal(signal.SIGTERM, signal_handler)

def main():
    ensure_dir(OUTPUT_DIR)
    log("Starting Audio Recorder...")
    
    # Check for PyAudio
    has_pyaudio = False
    try:
        import pyaudio
        import wave
        has_pyaudio = True
    except ImportError:
        log("[WARN] PyAudio not installed. Entering DUMMY MODE (Sleeping). Install: pip install pyaudio")
        # Do not exit. Just sleep to keep the process alive so Agent doesn't crash-loop.
        has_pyaudio = False

    if has_pyaudio:
        p = pyaudio.PyAudio()

        try:
            # List devices (optional debug)
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
                # Generate filename
                timestamp = datetime.datetime.now().strftime("%Y%m%d_%H%M%S")
                filename = os.path.join(OUTPUT_DIR, f"vlog_audio_{timestamp}.wav")
                
                log(f"Recording to {filename} for {RECORD_SECONDS} seconds...")
                
                wf = wave.open(filename, 'wb')
                wf.setnchannels(CHANNELS)
                wf.setsampwidth(p.get_sample_size(pyaudio.paInt16))
                wf.setframerate(SAMPLE_RATE)

                start_time = time.time()
                try:
                    while time.time() - start_time < RECORD_SECONDS:
                        data = stream.read(int(SAMPLE_RATE * CHUNK_DURATION_MS / 1000), exception_on_overflow=False)
                        wf.writeframes(data)
                except Exception as e:
                    log(f"Error during recording loop: {e}")
                    traceback.print_exc()
                    break # Exit loop to restart process
                finally:
                    wf.close()
                    log(f"Finished {filename}")

        except Exception as e:
            log(f"[FATAL] Audio recorder crashed: {e}")
            traceback.print_exc()
            sys.exit(1)
        finally:
            try:
                stream.stop_stream()
                stream.close()
                p.terminate()
            except:
                pass
    else:
        # Dummy Mode Loop
        while True:
            time.sleep(60)
            log("[DUMMY] Audio recorder active but idle (no pydeps).")

if __name__ == "__main__":
    main()
