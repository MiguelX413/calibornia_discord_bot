use anyhow::Result;
use serenity::all::{CommandInteraction, Context};
use tracing::warn;

use crate::{
    ids::{ADMIN, MOD},
    responses::respond_ephemeral,
};

pub(super) async fn require_staff(ctx: &Context, command: &CommandInteraction) -> Result<bool> {
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
