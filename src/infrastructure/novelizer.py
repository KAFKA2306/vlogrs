from pathlib import Path

import google.generativeai as genai

from src.infrastructure.settings import settings


class Novelizer:
    def __init__(self):
        self._model = None
        self._prompt_template = (
            Path(__file__).with_name("novelizer_prompt.txt").read_text(encoding="utf-8")
        )

    def generate_chapter(
        self,
        today_summary: str,
        novel_so_far: str = "",
    ) -> str:
        if not self._model:
            genai.configure(api_key=settings.gemini_api_key)
            self._model = genai.GenerativeModel(settings.novel_model)

        prompt = self._prompt_template.format(
            novel_so_far=novel_so_far,
            today_summary=today_summary,
        )

        response = self._model.generate_content(
            prompt,
            generation_config={"max_output_tokens": settings.novel_max_output_tokens},
        )
        return response.text.strip()
