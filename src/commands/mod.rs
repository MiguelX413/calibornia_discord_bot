mod auth;
mod members;
mod message;
mod options;
mod reactions;
mod registration;
mod verification;

use anyhow::Result;
use serenity::all::{CommandInteraction, Context};

pub(crate) use registration::register_commands;
pub(crate) use verification::handle_component;

use crate::commands::{
    auth::require_staff,
    members::{handle_list_unverified, handle_member_count},
    message::handle_message_command,
    reactions::{handle_poll, handle_unreact},
    verification::{handle_message_verify, handle_user_verify},
};

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
