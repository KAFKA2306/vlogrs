$root = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $root
$env:UV_PROJECT_ENVIRONMENT = ".venv-win"
$env:UV_LINK_MODE = "copy"
$env:PYTHONIOENCODING = "utf-8"
uv run python -m src.main
