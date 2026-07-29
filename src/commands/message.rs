use anyhow::{Context as _, Result, anyhow};
use serenity::all::{CommandInteraction, Context, CreateMessage, Message, ResolvedValue};

use crate::{
    commands::options::{channel_option, string_option, user_option},
    embeds::{sent_message_embed, target_label_user},
    ids::DAVEBOT,
    responses::respond_ephemeral,
};

pub(super) async fn handle_message_command(
    ctx: &Context,
    command: &CommandInteraction,
) -> Result<()> {
    if command.channel_id != DAVEBOT {
        respond_ephemeral(
            ctx,
            command,
            &format!("U gotta run this command in <#{}>", DAVEBOT.get()),
        )
        .await?;
        return Ok(());
    }

    let subcommand = command
        .data
        .options()
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("missing message subcommand"))?;

    let ResolvedValue::SubCommand(options) = subcommand.value else {
        return Err(anyhow!("message command option was not a subcommand"));
    };

    let message_text = string_option(&options, "message")?;
    match subcommand.name {
        "user" => {
            let user = user_option(&options, "user")?;
            if user.id == ctx.cache.current_user().id {
                respond_ephemeral(ctx, command, "Lmao why did u try to message me").await?;
                return Ok(());
            }
            let sent = user
                .direct_message(&ctx.http, CreateMessage::new().content(message_text))
                .await
                .context("sending user DM")?;
            log_sent_message(ctx, command, &sent, &target_label_user(user)).await?;
        }
        "channel" => {
            let channel = channel_option(&options, "channel")?;
            let sent = channel
                .id
                .send_message(&ctx.http, CreateMessage::new().content(message_text))
                .await
                .context("sending channel message")?;
            let target = channel
                .name
                .as_ref()
                .map(|name| format!("{name} (<#{}>)", channel.id.get()))
                .unwrap_or_else(|| format!("<#{}>", channel.id.get()));
            log_sent_message(ctx, command, &sent, &target).await?;
        }
        name => return Err(anyhow!("unknown message subcommand {name}")),
    }

    respond_ephemeral(ctx, command, "Sent").await?;
    Ok(())
}

async fn log_sent_message(
    ctx: &Context,
    command: &CommandInteraction,
    sent: &Message,
    target: &str,
) -> Result<()> {
    DAVEBOT
        .send_message(
            &ctx.http,
            CreateMessage::new().embed(sent_message_embed(sent, &command.user, target)),
        )
        .await
        .context("logging sent message")?;
    Ok(())
}
