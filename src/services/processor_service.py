import logging
from pathlib import Path

from src.domain.entities import RecordingSession
from src.infrastructure.preprocessor import TranscriptPreprocessor
from src.infrastructure.summarizer import Summarizer
from src.infrastructure.transcriber import Transcriber
from src.sync_supabase import main as sync_supabase

logger = logging.getLogger(__name__)


class ProcessorService:
    def __init__(
        self,
        transcriber: Transcriber,
        summarizer: Summarizer,
        preprocessor: TranscriptPreprocessor,
    ):
        self._transcriber = transcriber
        self._summarizer = summarizer
        self._preprocessor = preprocessor

    def process_session(self, session: RecordingSession) -> str:
        logger.info(f"Processing session: {session}")
        logger.info("Transcribing audio...")
        transcript, transcript_path = self._transcriber.transcribe_and_save(
            session.file_path
        )
        self._transcriber.unload()

        logger.info("Preprocessing transcript...")
        cleaned_transcript = self._preprocessor.process(transcript)

        cleaned_path = Path(transcript_path).with_name(
            f"cleaned_{Path(transcript_path).name}"
        )
        cleaned_path.write_text(cleaned_transcript, encoding="utf-8")
        logger.info(f"Cleaned transcript saved to {cleaned_path}")

        logger.info("Summarizing transcript...")
        summary = self._summarizer.summarize(cleaned_transcript, session)

        summary_path = Path("summaries") / f"{Path(session.file_path).stem}_summary.txt"
        summary_path.parent.mkdir(exist_ok=True)
        summary_path.write_text(summary, encoding="utf-8")

        logger.info(
            "Processing complete. Summary saved to %s",
            summary_path,
        )
        sync_supabase()
        return str(summary_path)
