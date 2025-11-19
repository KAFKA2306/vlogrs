import psutil

from src.infrastructure.settings import settings


class ProcessMonitor:
    def __init__(self):
        self._targets = {name.lower() for name in settings.process_names}

    def is_running(self) -> bool:
        for proc in psutil.process_iter(["name"]):
            name = proc.info.get("name")
            if name and name.lower() in self._targets:
                return True
        return False
