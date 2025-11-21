import os

import google.generativeai as genai

from src.domain.entities import RecordingSession
from src.infrastructure.settings import settings


class Summarizer:
    def __init__(self):
        self._model = None

    def summarize(self, transcript: str, session: RecordingSession) -> str:
        model = self._ensure_model()
        start_ts = session.start_time.strftime("%Y-%m-%d %H:%M")
        end_ts = (session.end_time or session.start_time).strftime("%Y-%m-%d %H:%M")
        prompt = (
            "あなたはVRChatプレイログから日記を書くアシスタントです。\n"
            "以下のルールでMarkdownテキストのみを日本語で出力してください。\n\n"
            "【出力形式】\
            "  ## VRChat {日付} {開始時刻}–{終了時刻}\n\n"
            "### 今日の出来事\n"
            "- 箇条書き3〜5個で出来事を要約（具体的な会話内容や感情も含める）\n"
            "- 誰と何をしたか、どう感じたかを書く\n\n"
            "---\n"
            "### 気づき\n"
            "1〜2行で今日の気づきや次にやりたいことを書く\n\n"
            "【注意事項】\n"
            "- 実名が含まれていたらイニシャルに置き換える\n"
            "- ない内容は作らない\n"
            "- 具体的なエピソードや会話内容を含めて、"
            "読み返したときに思い出せるように書く\n"
            "- 感情や印象も含めて、日記らしい温かみのある文章にする\n\n"
            f"セッション時間: {start_ts} 〜 {end_ts}\n"
            "---\n"
            f"{transcript.strip()}\n"
        )
        response = model.generate_content(prompt)
        return response.text.strip()

    def _ensure_model(self):
        if self._model:
            return self._model
        api_key = settings.gemini_api_key or os.getenv(settings.gemini_api_key_env)
        genai.configure(api_key=api_key)
        self._model = genai.GenerativeModel(settings.gemini_model)
        return self._model
