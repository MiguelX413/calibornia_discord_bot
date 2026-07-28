# calibornia_discord_bot

Discord bot for the Calibornia Discord.

This repository now uses `uv` for dependency management.

The `uv.lock` file should be committed for reproducible installs.

Current runtime:

- Python 3.14
- `py-cord`
- `emoji`

Setup:

- `uv sync --group dev`

Basic commands:

- `uv run python -m py_compile src/calibornia_discord_bot/*.py`
- `uv run ruff check .`
- `uv run ruff format --check .`
- `uv run mypy`
- `uv run python -m calibornia_discord_bot`

Run:

- `export TOKEN=...`
- `uv run python -m calibornia_discord_bot`

Notes:

- Runtime code now lives under `src/calibornia_discord_bot/`.
- The bot accepts `TOKEN` and still falls back to the older lowercase `token` environment variable.
- `pyproject.toml` and `uv.lock` are the source of truth for dependencies.
