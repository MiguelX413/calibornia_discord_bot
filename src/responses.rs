use anyhow::{Context as _, Result};
use serenity::all::{
    CommandInteraction, Context, CreateInteractionResponse, CreateInteractionResponseFollowup,
    CreateInteractionResponseMessage,
};

pub(crate) async fn respond_ephemeral(
    ctx: &Context,
    command: &CommandInteraction,
    content: &str,
) -> Result<()> {
    command
        .create_response(
            &ctx.http,
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .content(content)
                    .ephemeral(true),
            ),
        )
        .await
        .context("sending ephemeral response")?;
    Ok(())
}

pub(crate) async fn followup_ephemeral(
    ctx: &Context,
    command: &CommandInteraction,
    content: &str,
) -> Result<()> {
    command
        .create_followup(
            &ctx.http,
            CreateInteractionResponseFollowup::new()
                .content(content)
                .ephemeral(true),
        )
        .await
        .context("sending ephemeral follow-up")?;
    Ok(())
}
