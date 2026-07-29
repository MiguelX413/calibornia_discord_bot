use anyhow::{Context as _, Result};
use serenity::all::{CommandOptionType, CommandType, Context, CreateCommand, CreateCommandOption};

use crate::{
    commands::{
        LIST_UNVERIFIED_COMMAND, MEMBER_COUNT_COMMAND, MESSAGE_COMMAND, POLL_COMMAND,
        UNREACT_COMMAND, VERIFY_MESSAGE_COMMAND, VERIFY_USER_COMMAND,
    },
    ids::GUILD,
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
        CreateCommand::new(VERIFY_USER_COMMAND).kind(CommandType::User),
        CreateCommand::new(VERIFY_MESSAGE_COMMAND).kind(CommandType::Message),
        CreateCommand::new(LIST_UNVERIFIED_COMMAND).description("Lists unverified members"),
        CreateCommand::new(MEMBER_COUNT_COMMAND).description("The amount of non-bot members."),
        CreateCommand::new(POLL_COMMAND).kind(CommandType::Message),
        CreateCommand::new(UNREACT_COMMAND).kind(CommandType::Message),
    ]
}

fn message_command() -> CreateCommand {
    CreateCommand::new(MESSAGE_COMMAND)
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
