use anyhow::{Context as _, Result};
use serenity::all::{Context, CreateAttachment, CreateMessage, Message};

use crate::{
    embeds::new_message_embed,
    emoji::triggered_emojis,
    ids::{DAVEBOT, SPAM},
};

pub(crate) async fn forward_dm(ctx: &Context, message: &Message) -> Result<()> {
    if message.author.bot || message.guild_id.is_some() {
        return Ok(());
    }

    let mut files = Vec::with_capacity(message.attachments.len());
    for attachment in &message.attachments {
        let bytes = attachment
            .download()
            .await
            .with_context(|| format!("downloading attachment {}", attachment.id))?;
        files.push(CreateAttachment::bytes(bytes, attachment.filename.clone()));
    }

    let embed = new_message_embed(ctx, message);
    let sticker_ids: Vec<_> = message
        .sticker_items
        .iter()
        .map(|sticker| sticker.id)
        .collect();
    let builder = CreateMessage::new()
        .embed(embed.clone())
        .files(files.clone())
        .sticker_ids(sticker_ids);

    if let Err(err) = DAVEBOT.send_message(&ctx.http, builder).await {
        if message.sticker_items.is_empty() {
            return Err(err).context("forwarding DM");
        }

        let sticker_list = format!("{:?}", message.sticker_items);
        DAVEBOT
            .send_message(
                &ctx.http,
                CreateMessage::new()
                    .content(format!(
                        "Message contained sticker which cannot be sent here.\nStickers: {sticker_list}"
                    ))
                    .embed(embed)
                    .files(files),
            )
            .await
            .with_context(|| format!("forwarding DM without stickers after error: {err}"))?;
    }

    Ok(())
}

pub(crate) async fn react_to_triggers(ctx: &Context, message: &Message) -> Result<()> {
    if message.author.bot {
        return Ok(());
    }

    let emojis = triggered_emojis(&message.content);
    if message.channel_id == SPAM {
        let reply = emojis
            .iter()
            .map(|emoji| emoji.mention())
            .collect::<String>();
        if !reply.is_empty() {
            message
                .reply(&ctx.http, reply)
                .await
                .context("replying with emojis")?;
        }
        return Ok(());
    }

    for emoji in emojis {
        message
            .react(&ctx.http, emoji.reaction())
            .await
            .with_context(|| format!("reacting with {}", emoji.name))?;
    }

    Ok(())
}
