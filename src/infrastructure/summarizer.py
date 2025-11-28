from pathlib import Path

import google.generativeai as genai

from src.domain.entities import RecordingSession
from src.infrastructure.settings import settings


class Summarizer:
    def __init__(self):
        self._model = None
        self._prompt_template = (
            Path(__file__)
            .with_name("summarizer_prompt.txt")
            .read_text(encoding="utf-8")
        )

    def summarize(self, transcript: str, session: RecordingSession) -> str:
        if not self._model:
            genai.configure(api_key=settings.gemini_api_key)
            self._model = genai.GenerativeModel(settings.gemini_model)

        prompt = self._prompt_template.format(
            date=session.start_time.strftime("%Y-%m-%d"),
            start_time=session.start_time.strftime("%H:%M"),
            end_time=(session.end_time or session.start_time).strftime("%H:%M"),
            transcript=transcript.strip(),
        )
        response = self._model.generate_content(prompt)
        return response.text.strip()
