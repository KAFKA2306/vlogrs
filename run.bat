@echo off
pushd "%~dp0"
echo Starting VRChat Auto-Diary...
uv run python -m src.main
popd
pause
