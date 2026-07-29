mod auth;
mod members;
mod message;
mod options;
mod reactions;
mod registration;
mod verification;

use std::future::Future;

use anyhow::Result;
use serenity::all::{CommandInteraction, Context};

pub(crate) use registration::register_commands;
pub(crate) use verification::handle_component;

use crate::commands::{
    auth::require_staff,
    members::{handle_list_unverified, handle_member_count},
    message::handle_message_command,
    reactions::handle_reactions,
    verification::{handle_message_verify, handle_user_verify},
};

pub(super) const MESSAGE_COMMAND: &str = "message";
pub(super) const VERIFY_USER_COMMAND: &str = "Verify without intro";
pub(super) const VERIFY_MESSAGE_COMMAND: &str = "Verify";
pub(super) const LIST_UNVERIFIED_COMMAND: &str = "list_unverified";
pub(super) const MEMBER_COUNT_COMMAND: &str = "member_count";
pub(super) const POLL_COMMAND: &str = "Poll";
pub(super) const UNREACT_COMMAND: &str = "Unreact";

pub(crate) async fn handle_command(ctx: &Context, command: CommandInteraction) -> Result<()> {
    match command.data.name.as_str() {
        MESSAGE_COMMAND => staff_only(ctx, &command, handle_message_command(ctx, &command)).await,
        VERIFY_USER_COMMAND => staff_only(ctx, &command, handle_user_verify(ctx, &command)).await,
        VERIFY_MESSAGE_COMMAND => {
            staff_only(ctx, &command, handle_message_verify(ctx, &command)).await
        }
        LIST_UNVERIFIED_COMMAND => {
            staff_only(ctx, &command, handle_list_unverified(ctx, &command)).await
        }
        MEMBER_COUNT_COMMAND => handle_member_count(ctx, &command).await,
        POLL_COMMAND => handle_reactions(ctx, &command, true).await,
        UNREACT_COMMAND => handle_reactions(ctx, &command, false).await,
        _ => Ok(()),
    }
}

async fn staff_only(
    ctx: &Context,
    command: &CommandInteraction,
    handler: impl Future<Output = Result<()>>,
) -> Result<()> {
    if require_staff(ctx, command).await? {
        handler.await
    } else {
        Ok(())
    }
}
