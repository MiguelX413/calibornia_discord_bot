use anyhow::Result;
use serenity::all::{
    Context, EventHandler, GuildId, Interaction, Member, Message, Ready, User, async_trait,
};
use tracing::{error, info};

use crate::{
    commands::{handle_command, handle_component, register_commands},
    members::{member_joined, member_left},
    messaging::{forward_dm, react_to_triggers},
};

pub(crate) struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("connected as {}", ready.user.tag());

        if let Err(err) = register_commands(&ctx).await {
            error!(?err, "failed to register guild commands");
        }
    }

    async fn message(&self, ctx: Context, message: Message) {
        let forward = forward_dm(&ctx, &message);
        let react = react_to_triggers(&ctx, &message);

        let (forward_result, react_result) = tokio::join!(forward, react);
        if let Err(err) = forward_result {
            error!(?err, "failed to forward message");
        }
        if let Err(err) = react_result {
            error!(?err, "failed to react to message");
        }
    }

    async fn guild_member_addition(&self, ctx: Context, member: Member) {
        if let Err(err) = member_joined(&ctx, member).await {
            error!(?err, "failed to handle member join");
        }
    }

    async fn guild_member_removal(
        &self,
        ctx: Context,
        _guild_id: GuildId,
        user: User,
        member_data_if_available: Option<Member>,
    ) {
        if let Err(err) = member_left(&ctx, user, member_data_if_available).await {
            error!(?err, "failed to handle member removal");
        }
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        let result: Result<()> = match interaction {
            Interaction::Command(command) => handle_command(&ctx, command).await,
            Interaction::Component(component) => handle_component(&ctx, component).await,
            _ => Ok(()),
        };

        if let Err(err) = result {
            error!(?err, "failed to handle interaction");
        }
    }
}
