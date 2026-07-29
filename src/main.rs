mod commands;
mod config;
mod embeds;
mod emoji;
mod handler;
mod ids;
mod members;
mod messaging;
mod responses;
mod utils;

use anyhow::{Context as _, Result};
use serenity::{Client, all::GatewayIntents};

use crate::{config::Config, handler::Handler};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "calibornia_discord_bot=info,serenity=info".into()),
        )
        .init();

    let config = Config::from_env()?;
    let intents = GatewayIntents::GUILDS
        | GatewayIntents::GUILD_MEMBERS
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILD_MESSAGE_REACTIONS;
    let mut client = Client::builder(config.discord_token, intents)
        .event_handler(Handler)
        .await
        .context("creating Discord client")?;

    client.start().await.context("running Discord client")
}
