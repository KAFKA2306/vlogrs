import json
from typing import Any, Dict, List

import google.generativeai as genai

from src.infrastructure.settings import settings


class JulesClient:
    def __init__(self):
        if not settings.jules_api_key:
            raise ValueError("GOOGLE_JULES_API_KEY is not set in .env")
        
        genai.configure(api_key=settings.jules_api_key)
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
        - estimated_minutes: An integer estimate of time required (default to 15 if unknown).
        
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
                "estimated_minutes": 15
            }

    def chat(self, history: List[Dict[str, str]], message: str) -> str:
        """
        Simple chat interface for 'management' discussions.
        """
        # This could be expanded for a full chat loop
        chat = self._model.start_chat(history=history)
        response = chat.send_message(message)
        return response.text
