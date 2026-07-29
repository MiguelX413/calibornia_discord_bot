use anyhow::{Result, anyhow};
use serenity::all::{CommandInteraction, Context};

use crate::{
    ids::{GUILD, MEMBER},
    members::member_counts,
    responses::{followup_ephemeral, respond_ephemeral},
    utils::chunk_lines,
};

pub(super) async fn handle_list_unverified(
    ctx: &Context,
    command: &CommandInteraction,
) -> Result<()> {
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

pub(super) async fn handle_member_count(ctx: &Context, command: &CommandInteraction) -> Result<()> {
    let counts = member_counts(ctx)?;
    respond_ephemeral(
        ctx,
        command,
        &format!(
            "There are currently {} non-bot members out of {} in the server.",
            counts.non_bot, counts.total
        ),
    )
    .await
}
