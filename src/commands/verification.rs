use anyhow::{Context as _, Result, anyhow};
use serenity::all::{
    ButtonStyle, CommandInteraction, ComponentInteraction, Context, CreateActionRow, CreateButton,
    CreateInteractionResponse, CreateInteractionResponseMessage, CreateMessage, Mentionable,
    Message, ResolvedTarget, UserId,
};

use crate::{
    embeds::new_message_embed,
    emoji::THUMBSUPDIRK_EMOJI,
    ids::{MEMBER, MODLOG, UNVERIFIED},
    responses::respond_ephemeral,
};

pub(super) async fn handle_user_verify(ctx: &Context, command: &CommandInteraction) -> Result<()> {
    let Some(ResolvedTarget::User(user, _partial_member)) = command.data.target() else {
        return Err(anyhow!("verify without intro missing target user"));
    };

    let Some(guild_id) = command.guild_id else {
        return Err(anyhow!("verification command outside guild"));
    };
    let member = match guild_id.member(&ctx.http, user.id).await {
        Ok(member) => member,
        Err(_) => {
            respond_ephemeral(ctx, command, "User no longer in the server").await?;
            return Ok(());
        }
    };

    if member.user.id == ctx.cache.current_user().id {
        respond_ephemeral(ctx, command, "You can't verify me!").await?;
        return Ok(());
    }
    if member.roles.contains(&MEMBER) {
        respond_ephemeral(ctx, command, "User already verified").await?;
        return Ok(());
    }

    respond_with_verify_button(ctx, command, member.user.id).await
}

pub(super) async fn handle_message_verify(
    ctx: &Context,
    command: &CommandInteraction,
) -> Result<()> {
    let Some(ResolvedTarget::Message(message)) = command.data.target() else {
        return Err(anyhow!("verify command missing target message"));
    };
    verify_member(ctx, command, message.author.id, Some(message)).await
}

async fn respond_with_verify_button(
    ctx: &Context,
    command: &CommandInteraction,
    member_id: UserId,
) -> Result<()> {
    command
        .create_response(
            &ctx.http,
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .content("Are you sure you want to verify this user without an intro?")
                    .ephemeral(true)
                    .components(vec![CreateActionRow::Buttons(vec![
                        CreateButton::new(format!("verify_without_intro:{}", member_id.get()))
                            .label("Verify without intro")
                            .style(ButtonStyle::Primary),
                    ])]),
            ),
        )
        .await
        .context("sending verification confirmation")?;
    Ok(())
}

pub(crate) async fn handle_component(ctx: &Context, component: ComponentInteraction) -> Result<()> {
    let Some(member_id) = component
        .data
        .custom_id
        .strip_prefix("verify_without_intro:")
        .and_then(|id| id.parse::<u64>().ok())
        .map(UserId::new)
    else {
        return Ok(());
    };

    let Some(guild_id) = component.guild_id else {
        return Err(anyhow!("verification component outside guild"));
    };

    let member = match guild_id.member(&ctx.http, member_id).await {
        Ok(member) => member,
        Err(_) => {
            component
                .create_response(
                    &ctx.http,
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new()
                            .content("User no longer in the server")
                            .ephemeral(true),
                    ),
                )
                .await?;
            return Ok(());
        }
    };

    if member.roles.contains(&MEMBER) {
        component
            .create_response(
                &ctx.http,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                        .content("User already verified")
                        .ephemeral(true),
                ),
            )
            .await?;
        return Ok(());
    }

    let modlog = MODLOG.say(
        &ctx.http,
        format!(
            "{} verified {} without an intro",
            component.user.mention(),
            member.mention()
        ),
    );
    let response = component.create_response(
        &ctx.http,
        CreateInteractionResponse::Message(
            CreateInteractionResponseMessage::new()
                .content(THUMBSUPDIRK_EMOJI.mention())
                .ephemeral(true),
        ),
    );

    let (verification_result, modlog_result, response_result) =
        tokio::join!(apply_verification(ctx, &member), modlog, response);
    verification_result?;
    modlog_result.context("logging verification")?;
    response_result.context("responding to verification")?;

    Ok(())
}

async fn verify_member(
    ctx: &Context,
    command: &CommandInteraction,
    user_id: UserId,
    intro_message: Option<&Message>,
) -> Result<()> {
    let Some(guild_id) = command.guild_id else {
        return Err(anyhow!("verification command outside guild"));
    };

    if user_id == ctx.cache.current_user().id {
        respond_ephemeral(ctx, command, "You can't verify me!").await?;
        return Ok(());
    }

    let member = match guild_id.member(&ctx.http, user_id).await {
        Ok(member) => member,
        Err(_) => {
            respond_ephemeral(ctx, command, "User no longer in the server").await?;
            return Ok(());
        }
    };

    if member.roles.contains(&MEMBER) {
        respond_ephemeral(ctx, command, "User already verified").await?;
        return Ok(());
    }

    let Some(intro_message) = intro_message else {
        respond_with_verify_button(ctx, command, member.user.id).await?;
        return Ok(());
    };

    let verification_response = THUMBSUPDIRK_EMOJI.mention();
    let response = respond_ephemeral(ctx, command, &verification_response);
    let modlog = MODLOG.send_message(
        &ctx.http,
        CreateMessage::new()
            .content(format!(
                "{} verified {}",
                command.user.mention(),
                member.mention()
            ))
            .embed(new_message_embed(ctx, intro_message)),
    );
    let reaction = intro_message.react(&ctx.http, THUMBSUPDIRK_EMOJI.reaction());

    let (verification_result, response_result, modlog_result, reaction_result) =
        tokio::join!(apply_verification(ctx, &member), response, modlog, reaction);
    verification_result?;
    response_result.context("responding to verification")?;
    modlog_result.context("logging verification")?;
    reaction_result.context("reacting to intro")?;

    Ok(())
}

async fn apply_verification(ctx: &Context, member: &serenity::all::Member) -> Result<()> {
    let add_role = member.add_role(&ctx.http, MEMBER);
    let remove_role = member.remove_role(&ctx.http, UNVERIFIED);
    let dm = member.user.direct_message(
        &ctx.http,
        CreateMessage::new()
            .content("Congratulations, you're now verified! Welcome to the server!"),
    );

    let (add_result, remove_result, dm_result) = tokio::join!(add_role, remove_role, dm);
    add_result.context("adding member role")?;
    remove_result.context("removing unverified role")?;
    dm_result.context("sending verification DM")?;

    Ok(())
}
