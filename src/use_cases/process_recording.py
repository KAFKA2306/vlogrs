from datetime import datetime
from pathlib import Path

from src.domain.entities import RecordingSession
from src.domain.interfaces import (
    StorageProtocol,
    SummarizerProtocol,
    TranscriberProtocol,
    TranscriptPreprocessorProtocol,
)
from src.infrastructure.settings import settings


def archive_audio_file(audio_path: str) -> None:
    if not settings.archive_after_process:
        return

    archive_dir = Path(settings.archive_dir)
    archive_dir.mkdir(exist_ok=True)
    src = Path(audio_path)
    dst = archive_dir / src.name
    src.rename(dst)


class ProcessRecordingUseCase:
    def __init__(
        self,
        transcriber: TranscriberProtocol,
        preprocessor: TranscriptPreprocessorProtocol,
        summarizer: SummarizerProtocol,
        storage: StorageProtocol,
    ):
        self._transcriber = transcriber
        self._preprocessor = preprocessor
        self._summarizer = summarizer
        self._storage = storage

    def execute(self, audio_path: str) -> bool:
        if not Path(audio_path).exists():
            return False

        transcript, transcript_path = self._transcriber.transcribe_and_save(audio_path)
        self._transcriber.unload()

        basename = audio_path.split("/")[-1].split(".")[0]
        start_time = datetime.strptime(basename, "%Y%m%d_%H%M%S")
        session = RecordingSession(
            file_paths=(audio_path,),
            start_time=start_time,
            end_time=datetime.now(),
        )

        cleaned_transcript = self._preprocessor.process(transcript)
        cleaned_path = Path(transcript_path).with_name(
            f"cleaned_{Path(transcript_path).name}"
        )
        cleaned_path.write_text(cleaned_transcript, encoding="utf-8")

        self._summarizer.summarize(cleaned_transcript, session)
        self._storage.sync()
        archive_audio_file(audio_path)

        return True

    def execute_session(self, session: RecordingSession) -> None:
        transcripts = [
            self._transcriber.transcribe_and_save(path) for path in session.file_paths
        ]
        self._transcriber.unload()
        merged = " ".join(text for text, _ in transcripts)
        cleaned = self._preprocessor.process(merged)
        self._summarizer.summarize(cleaned, session)
        self._storage.sync()

        for audio_path in session.file_paths:
            archive_audio_file(audio_path)
