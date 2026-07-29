use std::env;

use anyhow::{Context as _, Result, ensure};

pub(crate) struct Config {
    pub(crate) discord_token: String,
}

impl Config {
    pub(crate) fn from_env() -> Result<Self> {
        let discord_token = env::var("DISCORD_TOKEN")
            .or_else(|_| env::var("TOKEN"))
            .context("missing Discord token; set DISCORD_TOKEN or TOKEN")?;
        ensure!(
            !discord_token.trim().is_empty(),
            "Discord token environment variable must not be empty"
        );

        Ok(Self { discord_token })
    }
}
