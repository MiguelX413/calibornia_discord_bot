use anyhow::{Context as _, Result, anyhow};
use serenity::all::{
    ButtonStyle, CommandInteraction, ComponentInteraction, Context, CreateActionRow, CreateButton,
    CreateInteractionResponse, CreateInteractionResponseMessage, CreateMessage, GuildId, Member,
    Mentionable, Message, ResolvedTarget, UserId,
};
use tracing::warn;

use crate::{
    embeds::new_message_embed,
    emoji::THUMBSUPDIRK_EMOJI,
    ids::{MEMBER, MODLOG, UNVERIFIED},
    responses::{respond_component_ephemeral, respond_ephemeral},
};

const VERIFY_WITHOUT_INTRO_PREFIX: &str = "verify_without_intro:";

pub(super) async fn handle_user_verify(ctx: &Context, command: &CommandInteraction) -> Result<()> {
    let Some(ResolvedTarget::User(user, _partial_member)) = command.data.target() else {
        return Err(anyhow!("verify without intro missing target user"));
    };

    let Some(member) = command_verification_candidate(ctx, command, user.id).await? else {
        return Ok(());
    };

    respond_with_verify_button(ctx, command, member.user.id).await
}

pub(super) async fn handle_message_verify(
    ctx: &Context,
    command: &CommandInteraction,
) -> Result<()> {
    let Some(ResolvedTarget::Message(message)) = command.data.target() else {
        return Err(anyhow!("verify command missing target message"));
    };
    let Some(member) = command_verification_candidate(ctx, command, message.author.id).await?
    else {
        return Ok(());
    };

    verify_from_intro(ctx, command, &member, message).await
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
                        CreateButton::new(verification_component_id(member_id))
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
    let Some(member_id) = parse_verification_component_id(&component.data.custom_id) else {
        return Ok(());
    };

    let Some(guild_id) = component.guild_id else {
        return Err(anyhow!("verification component outside guild"));
    };

    let member = match verification_candidate(ctx, guild_id, member_id).await {
        Ok(member) => member,
        Err(message) => {
            respond_component_ephemeral(ctx, &component, message).await?;
            return Ok(());
        }
    };

    apply_verification(ctx, &member).await?;

    let response_text = THUMBSUPDIRK_EMOJI.mention();
    let response = respond_component_ephemeral(ctx, &component, &response_text);
    let modlog = MODLOG.say(
        &ctx.http,
        format!(
            "{} verified {} without an intro",
            component.user.mention(),
            member.mention()
        ),
    );
    let (response_result, modlog_result) = tokio::join!(response, modlog);

    if let Err(err) = modlog_result {
        warn!(?err, user = %member.user.id, "failed to log verification");
    }
    response_result.context("responding to verification")?;

    Ok(())
}

async fn verification_candidate(
    ctx: &Context,
    guild_id: GuildId,
    user_id: UserId,
) -> std::result::Result<Box<Member>, &'static str> {
    if user_id == ctx.cache.current_user().id {
        return Err("You can't verify me!");
    }

    let Ok(member) = guild_id.member(&ctx.http, user_id).await else {
        return Err("User no longer in the server");
    };

    if member.roles.contains(&MEMBER) {
        Err("User already verified")
    } else {
        Ok(Box::new(member))
    }
}

async fn command_verification_candidate(
    ctx: &Context,
    command: &CommandInteraction,
    user_id: UserId,
) -> Result<Option<Box<Member>>> {
    let Some(guild_id) = command.guild_id else {
        return Err(anyhow!("verification command outside guild"));
    };

    match verification_candidate(ctx, guild_id, user_id).await {
        Ok(member) => Ok(Some(member)),
        Err(message) => {
            respond_ephemeral(ctx, command, message).await?;
            Ok(None)
        }
    }
}

fn verification_component_id(member_id: UserId) -> String {
    format!("{VERIFY_WITHOUT_INTRO_PREFIX}{}", member_id.get())
}

fn parse_verification_component_id(custom_id: &str) -> Option<UserId> {
    custom_id
        .strip_prefix(VERIFY_WITHOUT_INTRO_PREFIX)?
        .parse()
        .ok()
        .map(UserId::new)
}

async fn verify_from_intro(
    ctx: &Context,
    command: &CommandInteraction,
    member: &Member,
    intro_message: &Message,
) -> Result<()> {
    apply_verification(ctx, member).await?;

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

    let (response_result, modlog_result, reaction_result) =
        tokio::join!(response, modlog, reaction);
    if let Err(err) = modlog_result {
        warn!(?err, user = %member.user.id, "failed to log verification");
    }
    if let Err(err) = reaction_result {
        warn!(?err, user = %member.user.id, "failed to react to intro");
    }
    response_result.context("responding to verification")?;

    Ok(())
}

async fn apply_verification(ctx: &Context, member: &Member) -> Result<()> {
    member
        .add_role(&ctx.http, MEMBER)
        .await
        .context("adding member role")?;
    member
        .remove_role(&ctx.http, UNVERIFIED)
        .await
        .context("removing unverified role")?;

    if let Err(err) = member
        .user
        .direct_message(
            &ctx.http,
            CreateMessage::new()
                .content("Congratulations, you're now verified! Welcome to the server!"),
        )
        .await
    {
        warn!(?err, user = %member.user.id, "failed to send verification DM");
    }

    Ok(())
}
