# calibornia_discord_bot

Discord bot for the Calibornia Discord.

Current runtime:

- Python 3.14
- `py-cord`
- `emoji`

Setup with `pip`:

- `python -m venv .venv`
- `source .venv/bin/activate`
- `python -m pip install -U pip`
- `python -m pip install -r requirements.txt`

Run:

- `export TOKEN=...`
- `python bot.py`

Notes:

- The bot accepts `TOKEN` and still falls back to the older lowercase `token` environment variable.
- Dependency pins in `requirements.txt` were updated to current releases on July 24, 2026.
