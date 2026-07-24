import os
from dataclasses import dataclass


@dataclass(frozen=True)
class BotConfig:
    token: str


def load_secret(name: str, default: str | None = None) -> str | None:
    value = os.getenv(name)
    if value:
        return value
    try:
        import secrets as local_secrets
    except ImportError:
        return default
    return getattr(local_secrets, name, default)


def load_config() -> BotConfig:
    token = load_secret("TOKEN") or load_secret("token")
    if not token:
        raise RuntimeError(
            "Missing TOKEN. Set it in the environment or provide secrets.py."
        )
    return BotConfig(token=token)
