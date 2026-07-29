use anyhow::{Context as _, Result, anyhow};
use serenity::all::{
    CommandInteraction, Context, EditInteractionResponse, Message, ResolvedTarget,
};

use crate::{
    emoji::{THUMBSUPDIRK_EMOJI, ordered_poll_emojis},
    responses::respond_ephemeral,
};

pub(super) async fn handle_poll(ctx: &Context, command: &CommandInteraction) -> Result<()> {
    let Some(ResolvedTarget::Message(message)) = command.data.target() else {
        return Err(anyhow!("poll command missing target message"));
    };

    respond_ephemeral(ctx, command, "Removing reactions...").await?;
    clear_bot_reactions(ctx, message).await?;
    command
        .edit_response(
            &ctx.http,
            EditInteractionResponse::new().content("Reacting..."),
        )
        .await?;

    for emoji in ordered_poll_emojis(&message.content) {
        message.react(&ctx.http, emoji).await?;
    }

    command
        .edit_response(
            &ctx.http,
            EditInteractionResponse::new()
                .content(format!("Done {}", THUMBSUPDIRK_EMOJI.mention())),
        )
        .await?;
    Ok(())
}

pub(super) async fn handle_unreact(ctx: &Context, command: &CommandInteraction) -> Result<()> {
    let Some(ResolvedTarget::Message(message)) = command.data.target() else {
        return Err(anyhow!("unreact command missing target message"));
    };

    respond_ephemeral(ctx, command, "Removing reactions...").await?;
    clear_bot_reactions(ctx, message).await?;
    command
        .edit_response(
            &ctx.http,
            EditInteractionResponse::new()
                .content(format!("Done {}", THUMBSUPDIRK_EMOJI.mention())),
        )
        .await?;
    Ok(())
}

async fn clear_bot_reactions(ctx: &Context, message: &Message) -> Result<()> {
    let bot_user_id = ctx.cache.current_user().id;
    for reaction in &message.reactions {
        message
            .delete_reaction(&ctx.http, Some(bot_user_id), reaction.reaction_type.clone())
            .await
            .with_context(|| format!("clearing reaction {:?}", reaction.reaction_type))?;
    }
    Ok(())
}
