import re
from datetime import datetime
from pathlib import Path

import matplotlib.pyplot as plt
import numpy as np
from typing_extensions import TYPE_CHECKING

from src.infrastructure.settings import settings

if TYPE_CHECKING:
    from faster_whisper import WhisperModel

logger = logging.getLogger(__name__)

SILENCE_THRESHOLD = 0.02


    FILLERS = [
        r"えー",
        r"あのー",
        r"うーん",
        r"えっと",
        r"なんて",
        r"まあ",
        r"そうですね",
        r"あー",
        r"んー",
        r"うん",
        r"ふん",
        r"あ",
        r"はは",
        r"ははは",
        r"なんか",
        r"え",
        r"お",
        r"ふんふん",
        r"ふんふんふん",
        r"うんうん",
        r"うんうんうん",
        r"はいはい",
        r"はいはいはい",
        r"はいはいはいはい",
        r"おー",
        r"ああ",
        r"んふん",
        r"そっか",
        r"そっかぁ",
        r"そうか",
        r"そうなんだ",
        r"えへへ",
        r"あの",
        r"あのね",
        r"あのさ",
        r"ん",
    ]

    def process(self, txt: str) -> str:
        txt = self._normalize_text(txt)
        txt = self._remove_repetition(txt)
        txt = self._remove_fillers(txt)
        txt = self._dedupe_words(txt)
        txt = self._merge_lines(txt)
        return txt

    def _normalize_text(self, txt: str) -> str:
        txt = txt.replace("…", " ")
        txt = re.sub(r"\.{2,}", " ", txt)
        return txt

    def _remove_repetition(self, txt: str) -> str:
        return re.sub(r"(.{1,4}?)\1{4,}", r"\1", txt)

    def _remove_fillers(self, txt: str) -> str:
        fillers = sorted(self.FILLERS, key=len, reverse=True)
        pattern_str = "|".join(fillers)
        pattern = f"(^|[\\s、。?!])({pattern_str})(?=[\\s、。?!]|$)"

        def repl(match: re.Match[str]) -> str:
            leading = match.group(1)
            return (leading if leading != "^" else "") + " "

        for _ in range(20):
            prev_txt = txt
            txt = re.sub(pattern, repl, txt)
            if txt == prev_txt:
                break

        txt = re.sub(r"\s+", " ", txt).strip()
        txt = re.sub(r"([、。])\1+", r"\1", txt)
        txt = re.sub(r"^[、。]+", "", txt).strip()
        txt = re.sub(r"\s+[、。]+", "", txt)
        txt = re.sub(r"\s+", " ", txt).strip()
        return txt

    def _dedupe_words(self, txt: str) -> str:
        return re.sub(r"(\S+)\s+\1\b", r"\1", txt)

    def _merge_lines(self, txt: str) -> str:
        txt = txt.replace("\n", " ")
        txt = re.sub(r"\s+", " ", txt).strip()
        return txt
