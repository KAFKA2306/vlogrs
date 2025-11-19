import threading
import time

from src.infrastructure.audio_recorder import AudioRecorder
from src.infrastructure.diary_writer import DiaryWriter
from src.infrastructure.process_monitor import ProcessMonitor
from src.infrastructure.settings import settings
from src.infrastructure.summarizer import Summarizer
from src.infrastructure.transcriber import Transcriber
from src.services.processor_service import ProcessorService
from src.services.recorder_service import RecorderService


class Application:
    def __init__(self):
        self._monitor = ProcessMonitor()
        self._recorder_service = RecorderService(AudioRecorder())
        self._processor_service = ProcessorService(
            Transcriber(),
            Summarizer(),
            DiaryWriter(),
        )

    def run(self):
        try:
            while True:
                self._tick()
                time.sleep(settings.check_interval)
        except KeyboardInterrupt:
            self._shutdown()

    def _tick(self):
        running = self._monitor.is_running()
        active_session = self._recorder_service.active_session
        if running and active_session is None:
            self._recorder_service.start_session()
            return
        if not running and active_session is not None:
            session = self._recorder_service.stop_session()
            if session:
                threading.Thread(
                    target=self._processor_service.process_session,
                    args=(session,),
                    daemon=True,
                ).start()

    def _shutdown(self):
        session = self._recorder_service.stop_session()
        if session:
            self._processor_service.process_session(session)
