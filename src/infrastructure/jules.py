import json
from typing import Any, Dict, List

import google.generativeai as genai

from src.infrastructure.settings import settings


class JulesClient:
    def __init__(self):
        # Prioritize a valid-looking API key (starts with AIza)
        jules_key = settings.jules_api_key
        gemini_key = settings.gemini_api_key

        api_key = None
        if jules_key and jules_key.startswith("AIza"):
            api_key = jules_key
        elif gemini_key and gemini_key.startswith("AIza"):
            api_key = gemini_key
        else:
            api_key = jules_key or gemini_key

        if not api_key:
            raise ValueError("Neither GOOGLE_JULES_API_KEY nor GOOGLE_API_KEY is set")

        genai.configure(api_key=api_key)
        self._model = genai.GenerativeModel(settings.jules_model)

    def parse_task(self, user_input: str) -> Dict[str, Any]:
        """
        Parses user input into a structured task using Gemini.
        """
        prompt = f"""
        You are Jules, a personal task management assistant. 
        Analyze the following user input and extract a structured task.
        
        User Input: "{user_input}"
        
        Return a JSON object with the following fields:
        - title: A concise summary of the task.
        - description: A more detailed description (if available).
        - priority: "high", "medium", or "low".
        - tags: A list of relevant tags (e.g., ["code", "chore", "urgent"]).
        - estimated_minutes: An integer estimate of time required (default to 15 
          if unknown).
        
        Output ONLY the JSON object. No markdown code blocks.
        """

        response = self._model.generate_content(prompt)
        try:
            # Strip markdown if present
            text = response.text.strip()
            if text.startswith("```json"):
                text = text[7:-3]
            elif text.startswith("```"):
                text = text[3:-3]
            return json.loads(text)
        except json.JSONDecodeError:
            # Fallback if JSON parsing fails
            return {
                "title": user_input,
                "description": "",
                "priority": "medium",
                "tags": [],
                "estimated_minutes": 15,
            }

    def chat(self, history: List[Dict[str, str]], message: str) -> str:
        """
        Simple chat interface for 'management' discussions.
        """
        # This could be expanded for a full chat loop
        chat = self._model.start_chat(history=history)
        response = chat.send_message(message)
        return response.text

    def generate_image_prompt(self, chapter_text: str) -> str:
        """
        Generates an image prompt from novel text using a stored template.
        """
        from pathlib import Path

        base_path = Path(__file__).parent
        prompt_template = (
            (base_path / "image_generator_gemini_prompt.txt")
            .read_text(encoding="utf-8")
            .strip()
        )

        prompt = prompt_template.format(chapter_text=chapter_text[:2000])

        response = self._model.generate_content(prompt)
        return response.text.strip()
