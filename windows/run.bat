@echo off
cd /d "%~dp0.."

set "UV_PROJECT_ENVIRONMENT=.venv-win"
set "UV_LINK_MODE=copy"
set "PYTHONIOENCODING=utf-8"

if not exist "data" mkdir "data"
if not exist "data\recordings" mkdir "data\recordings"
if not exist "data\transcripts" mkdir "data\transcripts"
if not exist "logs" mkdir "logs"

uv run python -m src.main
