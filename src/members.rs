use anyhow::{Context as _, Result, anyhow};
use serenity::all::{Context, Member, Mentionable, User};
use tracing::warn;

use crate::{
    emoji::VRISKA_EMOJI,
    ids::{DEFAULT_JOIN_ROLES, GENERAL, GUILD, INTROS, WELCOME_AND_RULES},
};

pub(crate) async fn member_joined(ctx: &Context, member: Member) -> Result<()> {
    let member_count = member_counts(ctx)?.non_bot;
    let welcome_msg = format!(
        "Welcome to hell, {}! We now number {member_count}! Check out <#{}> and <#{}> to get verified and check out <id:customize> to get roles!",
        member.mention(),
        WELCOME_AND_RULES.get(),
        INTROS.get(),
    );
    let channel_msg = if member_count == 413 {
        format!("{} is number 413... the holy number...", member.mention())
    } else {
        welcome_msg.clone()
    };

    member
        .add_roles(&ctx.http, DEFAULT_JOIN_ROLES)
        .await
        .context("adding default roles")?;

    let (channel_result, dm_result) = tokio::join!(
        GENERAL.say(&ctx.http, channel_msg),
        member.user.direct_message(
            &ctx.http,
            serenity::all::CreateMessage::new().content(welcome_msg),
        )
    );
    channel_result.context("sending welcome message")?;
    if let Err(err) = dm_result {
        warn!(?err, user = %member.user.id, "failed to send welcome DM");
    }

    Ok(())
}

pub(crate) async fn member_left(
    ctx: &Context,
    user: User,
    _member_data_if_available: Option<Member>,
) -> Result<()> {
    let count = member_counts(ctx)?.non_bot;

    GENERAL
        .say(
            &ctx.http,
            format!(
                "{} {} couldn't bear the torture. Our population lowers to {count}. They'll be back.",
                VRISKA_EMOJI.mention(),
                user.mention()
            ),
        )
        .await
        .context("sending leave message")?;
    Ok(())
}

pub(crate) struct MemberCounts {
    pub(crate) non_bot: usize,
    pub(crate) total: usize,
}

pub(crate) fn member_counts(ctx: &Context) -> Result<MemberCounts> {
    let guild = GUILD
        .to_guild_cached(&ctx.cache)
        .ok_or_else(|| anyhow!("guild is not cached"))?;
    let total = guild.members.len();
    let non_bot = guild
        .members
        .values()
        .filter(|member| !member.user.bot)
        .count();
    Ok(MemberCounts { non_bot, total })
}
