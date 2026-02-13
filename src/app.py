import logging
import time
from datetime import datetime

from src.domain.entities import RecordingSession
from src.infrastructure.ai import Summarizer
from src.infrastructure.repositories import FileRepository, SupabaseRepository
from src.infrastructure.settings import settings
from src.infrastructure.system import (
    AudioRecorder,
    ProcessMonitor,
    TaskRepository,
    Transcriber,
    TranscriptPreprocessor,
)
from src.use_cases.process_recording import ProcessRecordingUseCase

logger = logging.getLogger(__name__)


class Application:
    def __init__(self):
        self._monitor = ProcessMonitor()
        self._recorder = AudioRecorder()
        self._use_case = ProcessRecordingUseCase(
            transcriber=Transcriber(),
            preprocessor=TranscriptPreprocessor(),
            summarizer=Summarizer(),
            storage=SupabaseRepository(),
            file_repository=FileRepository(),
        )
        self._active_session = None

    def run(self):
        logger.info("Application started with Task-Driven Architecture")
        while True:
            try:
                self._tick()
                self._work()
            except Exception as e:
                logger.error(f"Critical error in main loop: {e}")
                raise  # Crash-on-fail
            time.sleep(settings.check_interval)

    def _tick(self):
        running = self._monitor.is_running()
        if running and not self._active_session:
            logger.info("VRChat process detected. Starting recording session.")
            self._active_session = self._recorder.start()
        elif not running and self._active_session:
            logger.info("VRChat process ended. Stopping recording.")
            file_paths = self._recorder.stop()
            self._active_session = None
            if file_paths:
                tasks = TaskRepository()
                tasks.add(
                    {
                        "type": "process_session",
                        "file_paths": list(file_paths),
                        "start_time": datetime.now().isoformat(),
                    }
                )

    def _work(self):
        tasks = TaskRepository()
        runnable = tasks.list_runnable()
        if not runnable:
            return

        for task in runnable:
            task_id = task["id"]
            logger.info(f"Processing task {task_id} ({task['type']})")
            tasks.update_status(task_id, "processing")

            try:
                if task["type"] == "process_session":
                    session = RecordingSession(
                        file_paths=tuple(task["file_paths"]),
                        start_time=datetime.fromisoformat(task["start_time"]),
                        end_time=datetime.now(),
                    )
                    self._use_case.execute_session(session)
                tasks.complete(task_id)
                logger.info(f"Task {task_id} completed successfully")
            except Exception as e:
                logger.error(f"Task {task_id} failed: {e}")
                tasks.update_status(task_id, "failed", error=str(e))
                raise  # Crash-on-fail to ensure clean state on restart
