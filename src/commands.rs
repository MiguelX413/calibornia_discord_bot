use anyhow::{Context as _, Result, anyhow};
use serenity::all::{
    ButtonStyle, CommandInteraction, CommandOptionType, CommandType, ComponentInteraction, Context,
    CreateActionRow, CreateButton, CreateCommand, CreateCommandOption, CreateInteractionResponse,
    CreateInteractionResponseMessage, CreateMessage, EditInteractionResponse, Mentionable, Message,
    ResolvedTarget, ResolvedValue, User, UserId,
};
use tracing::warn;

use crate::{
    embeds::{new_message_embed, sent_message_embed, target_label_user},
    emoji::{THUMBSUPDIRK_EMOJI, ordered_poll_emojis},
    ids::{ADMIN, DAVEBOT, GUILD, MEMBER, MOD, MODLOG, UNVERIFIED},
    members::member_counts,
    responses::{followup_ephemeral, respond_ephemeral},
    utils::chunk_lines,
};

pub(crate) async fn register_commands(ctx: &Context) -> Result<()> {
    GUILD
        .set_commands(&ctx.http, commands())
        .await
        .context("registering commands")?;
    Ok(())
}

fn commands() -> Vec<CreateCommand> {
    vec![
        message_command(),
        CreateCommand::new("Verify without intro").kind(CommandType::User),
        CreateCommand::new("Verify").kind(CommandType::Message),
        CreateCommand::new("list_unverified").description("Lists unverified members"),
        CreateCommand::new("member_count").description("The amount of non-bot members."),
        CreateCommand::new("Poll").kind(CommandType::Message),
        CreateCommand::new("Unreact").kind(CommandType::Message),
    ]
}

fn message_command() -> CreateCommand {
    CreateCommand::new("message")
        .description("Sends messages as bot")
        .add_option(message_subcommand(
            "user",
            "Sends a DM as the bot",
            CommandOptionType::User,
            "user",
            "Target user",
        ))
        .add_option(message_subcommand(
            "channel",
            "Sends a channel message as the bot",
            CommandOptionType::Channel,
            "channel",
            "Target channel",
        ))
}

fn message_subcommand(
    name: &'static str,
    description: &'static str,
    target_kind: CommandOptionType,
    target_name: &'static str,
    target_description: &'static str,
) -> CreateCommandOption {
    CreateCommandOption::new(CommandOptionType::SubCommand, name, description)
        .add_sub_option(
            CreateCommandOption::new(target_kind, target_name, target_description).required(true),
        )
        .add_sub_option(
            CreateCommandOption::new(CommandOptionType::String, "message", "Message")
                .required(true),
        )
}

pub(crate) async fn handle_command(ctx: &Context, command: CommandInteraction) -> Result<()> {
    match command.data.name.as_str() {
        "message" => {
            if !require_staff(ctx, &command).await? {
                return Ok(());
            }
            handle_message_command(ctx, &command).await
        }
        "Verify without intro" => {
            if !require_staff(ctx, &command).await? {
                return Ok(());
            }
            handle_user_verify(ctx, &command).await
        }
        "Verify" => {
            if !require_staff(ctx, &command).await? {
                return Ok(());
            }
            handle_message_verify(ctx, &command).await
        }
        "list_unverified" => {
            if !require_staff(ctx, &command).await? {
                return Ok(());
            }
            handle_list_unverified(ctx, &command).await
        }
        "member_count" => handle_member_count(ctx, &command).await,
        "Poll" => handle_poll(ctx, &command).await,
        "Unreact" => handle_unreact(ctx, &command).await,
        _ => Ok(()),
    }
}

async fn require_staff(ctx: &Context, command: &CommandInteraction) -> Result<bool> {
    let Some(guild_id) = command.guild_id else {
        respond_ephemeral(ctx, command, "This command must run in a guild").await?;
        warn!(user = %command.user.id, command = %command.data.name, "staff command used outside guild");
        return Ok(false);
    };

    let member = match &command.member {
        Some(member) => member,
        None => {
            respond_ephemeral(ctx, command, "This command must run in a guild").await?;
            warn!(
                user = %command.user.id,
                guild = %guild_id,
                command = %command.data.name,
                "staff command missing member data"
            );
            return Ok(false);
        }
    };

    let is_staff = member
        .roles
        .iter()
        .any(|role| *role == MOD || *role == ADMIN);
    if !is_staff {
        respond_ephemeral(
            ctx,
            command,
            "You do not have permission to use this command",
        )
        .await?;
        warn!(
            user = %command.user.id,
            guild = %guild_id,
            command = %command.data.name,
            "non-staff user attempted staff command"
        );
        return Ok(false);
    }

    Ok(true)
}

async fn handle_message_command(ctx: &Context, command: &CommandInteraction) -> Result<()> {
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

async fn handle_user_verify(ctx: &Context, command: &CommandInteraction) -> Result<()> {
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

async fn handle_message_verify(ctx: &Context, command: &CommandInteraction) -> Result<()> {
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

async fn handle_list_unverified(ctx: &Context, command: &CommandInteraction) -> Result<()> {
    let lines = {
        let guild = GUILD
            .to_guild_cached(&ctx.cache)
            .ok_or_else(|| anyhow!("guild is not cached"))?;
        let mut members: Vec<_> = guild
            .members
            .values()
            .filter(|member| !member.user.bot && !member.roles.contains(&MEMBER))
            .filter_map(|member| member.joined_at.map(|joined| (member.user.id, joined)))
            .collect();
        members.sort_by_key(|(_, joined)| *joined);
        members
            .into_iter()
            .map(|(id, joined)| format!("<@{}>: <t:{}:f>", id.get(), joined.unix_timestamp()))
            .collect::<Vec<_>>()
    };

    let chunks = chunk_lines(lines, 2000);
    if chunks.is_empty() {
        respond_ephemeral(ctx, command, "No unverified members.").await?;
        return Ok(());
    }

    let mut chunks = chunks.into_iter();
    if let Some(first) = chunks.next() {
        respond_ephemeral(ctx, command, &first).await?;
    }
    for chunk in chunks {
        followup_ephemeral(ctx, command, &chunk).await?;
    }
    Ok(())
}

async fn handle_member_count(ctx: &Context, command: &CommandInteraction) -> Result<()> {
    let (non_bot, total) = member_counts(ctx)?;
    respond_ephemeral(
        ctx,
        command,
        &format!("There are currently {non_bot} non-bot members out of {total} in the server."),
    )
    .await
}

async fn handle_poll(ctx: &Context, command: &CommandInteraction) -> Result<()> {
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

async fn handle_unreact(ctx: &Context, command: &CommandInteraction) -> Result<()> {
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

fn string_option<'a>(
    options: &'a [serenity::all::ResolvedOption<'a>],
    name: &str,
) -> Result<&'a str> {
    options
        .iter()
        .find(|option| option.name == name)
        .and_then(|option| match option.value {
            ResolvedValue::String(value) => Some(value),
            _ => None,
        })
        .ok_or_else(|| anyhow!("missing string option {name}"))
}

fn user_option<'a>(
    options: &'a [serenity::all::ResolvedOption<'a>],
    name: &str,
) -> Result<&'a User> {
    options
        .iter()
        .find(|option| option.name == name)
        .and_then(|option| match option.value {
            ResolvedValue::User(user, _) => Some(user),
            _ => None,
        })
        .ok_or_else(|| anyhow!("missing user option {name}"))
}

fn channel_option<'a>(
    options: &'a [serenity::all::ResolvedOption<'a>],
    name: &str,
) -> Result<&'a serenity::all::PartialChannel> {
    options
        .iter()
        .find(|option| option.name == name)
        .and_then(|option| match option.value {
            ResolvedValue::Channel(channel) => Some(channel),
            _ => None,
        })
        .ok_or_else(|| anyhow!("missing channel option {name}"))
}
