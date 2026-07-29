use anyhow::{Context as _, Result};
use serenity::all::{CommandOptionType, CommandType, Context, CreateCommand, CreateCommandOption};

use crate::ids::GUILD;

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
