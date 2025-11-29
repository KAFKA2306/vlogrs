import json
import uuid
from datetime import datetime
from pathlib import Path
from typing import Any, Dict, List


class TaskRepository:
    def __init__(self, file_path: str = "data/tasks.json"):
        self.file_path = Path(file_path)
        self.file_path.parent.mkdir(parents=True, exist_ok=True)
        if not self.file_path.exists():
            self.file_path.write_text("[]", encoding="utf-8")

    def _load(self) -> List[Dict[str, Any]]:
        try:
            return json.loads(self.file_path.read_text(encoding="utf-8"))
        except json.JSONDecodeError:
            return []

    def _save(self, tasks: List[Dict[str, Any]]):
        self.file_path.write_text(
            json.dumps(tasks, indent=2, ensure_ascii=False), encoding="utf-8"
        )

    def add(self, task_data: Dict[str, Any]) -> Dict[str, Any]:
        tasks = self._load()
        new_task = {
            "id": str(uuid.uuid4()),
            "created_at": datetime.now().isoformat(),
            "status": "pending",
            **task_data,
        }
        tasks.append(new_task)
        self._save(tasks)
        return new_task

    def list_pending(self) -> List[Dict[str, Any]]:
        tasks = self._load()
        return [t for t in tasks if t.get("status") != "completed"]

    def complete(self, task_id_prefix: str) -> Dict[str, Any] | None:
        """
        Completes a task by ID or partial ID (first 8 chars).
        """
        tasks = self._load()
        found = False
        target_task = None

        for task in tasks:
            if task["id"].startswith(task_id_prefix):
                task["status"] = "completed"
                task["completed_at"] = datetime.now().isoformat()
                target_task = task
                found = True
                break

        if found:
            self._save(tasks)
            return target_task
        return None
