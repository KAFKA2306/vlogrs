#!/usr/bin/env python3
import re
from datetime import datetime
from pathlib import Path


def clean(line: str) -> str:
    return re.sub(r"^\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2},\d{3} ", "", line)


def main():
    log = Path("logs/vlog.log")
    if not log.exists():
        print("状態: ログファイルが見つかりません")
        return
    lines = [
        line for line in log.read_text(encoding="utf-8").splitlines() if line.strip()
    ]
    if not lines:
        print("状態: ログが空です")
        return

    state, last_session, start, action = "待機中", None, None, None
    warnings = []
    for line in reversed(lines[-100:]):
        if len(warnings) < 3 and ("WARNING" in line or "appears empty" in line):
            warnings.append(clean(line))
        if "VRChat process detected" in line:
            state = "録音中"
            if m := re.search(r"(\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2})", line):
                last_session = m.group(1)
                start = datetime.strptime(last_session, "%Y-%m-%d %H:%M:%S")
            break
        if "Transcribing audio" in line:
            state, action = "文字起こし中", "文字起こし処理中..."
            break
        if "Summarizing transcript" in line:
            state, action = "要約中", "要約処理中..."
            break
        if "Processing complete" in line or "daily summary" in line:
            state, action = "完了", "処理完了"
            break
        if "VRChat process ended" in line and not last_session:
            if m := re.search(r"(\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2})", line):
                last_session = m.group(1)
            state = "処理中"

    print(f"状態: {state}")
    if last_session:
        print(f"セッション: {last_session}")
    if start:
        mins = int((datetime.now() - start).total_seconds() / 60)
        hours, mins = divmod(mins, 60)
        print(f"録音時間: {hours}h {mins}m" if hours else f"録音時間: {mins}m")
    if action:
        print(f"進捗: {action}")
    if warnings:
        print("\n⚠️  警告:")
        for w in warnings:
            print(f"  - {w}")

    print("\n最新ログ:")
    for line in lines[-5:]:
        print(f"  {clean(line)}")


if __name__ == "__main__":
    main()
