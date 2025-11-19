import os

from src.domain.entities import DiaryEntry
from src.infrastructure.settings import settings


class DiaryWriter:
    def write(self, entry: DiaryEntry) -> str:
        os.makedirs(settings.diary_dir, exist_ok=True)
        filename = entry.date.strftime("%Y-%m-%d_vrchat.md")
        path = os.path.join(settings.diary_dir, filename)
        start = entry.session_start.strftime("%H:%M")
        end = entry.session_end.strftime("%H:%M")
        session_window = f"{start}–{end}"
        with open(path, "a", encoding="utf-8") as file:
            file.write(
                f"## VRChat {entry.date.strftime('%Y-%m-%d')} {session_window}\n\n"
            )
            file.write(f"{entry.summary.strip()}\n\n")
            file.write("### Transcript\n")
            file.write(f"{entry.raw_log.strip()}\n\n")
        return path
