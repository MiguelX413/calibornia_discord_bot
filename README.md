# calibornia_discord_bot

Discord bot for the Calibornia Discord.

This repository is now a Rust Cargo project using `serenity`.

Current runtime:

- Rust 2024 edition
- `serenity`
- `tokio`

Setup:

- Install a current Rust toolchain.
- Set `DISCORD_TOKEN` in the environment.

Basic commands:

- `cargo fmt --check`
- `cargo check`
- `cargo run`

Run:

- `export DISCORD_TOKEN=...`
- `cargo run`

Notes:

- Runtime code lives in `src/main.rs`.
- The bot reads `DISCORD_TOKEN`, with `TOKEN` kept as a compatibility fallback.
