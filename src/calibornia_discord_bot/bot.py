#!/usr/bin/env python3
import asyncio
import logging
from collections.abc import Callable, Iterable, Sequence
from datetime import UTC, datetime
from typing import Any

import discord
from discord.ext.commands import has_any_role
from discord.utils import format_dt
from emoji import emoji_list

from .config import load_config

GUILD = 980962249550213170

CHANNELS = {
    "intros": 980968056245354596,
    "welcome-and-rules": 980962249550213172,
    "roles": 981413078556086312,
    "general": 980962249550213176,
    "spam": 981995926883287142,
    "modlog": 981416669706608650,
    "davebot": 1089751694352584725,
}

ROLES = {
    "admin": 980964927164518470,
    "member": 982177726691700736,
    "mod": 1027089314405957685,
    "color_divider": 1027311103014862888,
    "location_divider": 1027310335314628708,
    "ping_divider": 1027095201262616607,
    "pronoun_divider": 1027094772848005160,
    "classpect_divider": 1027309033373310987,
    "misc_divider": 1027309906807750676,
    "unverified": 1098091859743612948,
}

JOIN_LEAVE_MSG_CHANNEL = CHANNELS["general"]

type MessageTarget = discord.abc.Messageable
type EmojiResolver = Callable[
    [], discord.Emoji | discord.PartialEmoji | discord.AppEmoji | None
]
type EmbedAuthor = discord.User | discord.Member
type PollEmoji = str | discord.Emoji | discord.AppEmoji


def _guild(bot: discord.Bot) -> discord.Guild:
    guild = bot.get_guild(GUILD)
    if guild is None:
        raise RuntimeError(f"Bot is not connected to guild {GUILD}")
    return guild


def _text_channel(bot: discord.Bot, channel_id: int) -> discord.TextChannel:
    channel = _guild(bot).get_channel(channel_id)
    if not isinstance(channel, discord.TextChannel):
        raise RuntimeError(f"Channel {channel_id} is not an available text channel")
    return channel


def _role(guild: discord.Guild, role_id: int) -> discord.Role:
    role = guild.get_role(role_id)
    if role is None:
        raise RuntimeError(f"Role {role_id} is not available")
    return role


def _emoji(name: str) -> discord.Emoji | discord.PartialEmoji | discord.AppEmoji:
    emoji = EMOJIS[name]()
    if emoji is None:
        raise RuntimeError(f"Emoji {name} is not available")
    return emoji


def _has_role(member: discord.Member, role_id: int) -> bool:
    return any(role.id == role_id for role in member.roles)


def _member_role(member: discord.Member, role_name: str) -> discord.Role:
    return _role(member.guild, ROLES[role_name])


def _set_embed_author(embed: discord.Embed, user: EmbedAuthor) -> None:
    embed.set_author(
        name=f"{user} ({user.mention})",
        url=f"https://discordapp.com/users/{user.id}",
        icon_url=user.display_avatar.url,
    )


def _message_reference_id(message: discord.Message) -> int | None:
    return message.reference.message_id if message.reference else None


def _message_author_color(message: discord.Message) -> discord.Colour:
    author = message.author
    if isinstance(author, discord.Member):
        return author.color
    return discord.Colour.default()


def _new_message_embed(message: discord.Message) -> discord.Embed:
    embed = discord.Embed(
        description=message.content,
        color=_message_author_color(message),
        timestamp=message.created_at,
    )
    embed.add_field(name="Message ID", value=str(message.id))
    embed.add_field(name="Channel ID", value=str(message.channel.id))
    if (reference_id := _message_reference_id(message)) is not None:
        embed.add_field(name="Reference", value=str(reference_id))
    _set_embed_author(embed, message.author)
    return embed


async def _on_message_forward(bot: discord.Bot, message: discord.Message) -> None:
    if bot.application_id == message.author.id or message.guild is not None:
        return

    embed = _new_message_embed(message)
    files = [
        await attachment.to_file(use_cached=True) for attachment in message.attachments
    ]
    channel = _text_channel(bot, CHANNELS["davebot"])
    try:
        await channel.send(embed=embed, files=files, stickers=message.stickers)
    except discord.errors.Forbidden:
        await channel.send(
            content=(
                "Message contained sticker which cannot be sent here.\n"
                f"Stickers: {message.stickers}"
            ),
            embed=embed,
            files=files,
        )


async def _on_message_react(bot: discord.Bot, message: discord.Message) -> None:
    if bot.application_id == message.author.id:
        return

    triggered_emojis = _triggered_emojis(message.content)
    if message.channel.id == CHANNELS["spam"]:
        reply = "".join(str(_emoji_by_factory(emoji)) for emoji in triggered_emojis)
        if reply:
            await message.reply(reply)
        return

    for emoji in triggered_emojis:
        await message.add_reaction(_emoji_by_factory(emoji))


class DaveBot(discord.Bot):
    async def on_message(self, message: discord.Message) -> None:
        await asyncio.gather(
            _on_message_forward(self, message),
            _on_message_react(self, message),
        )

    async def on_member_join(self, member: discord.Member) -> None:
        current_member_count = non_bot_member_count(member.guild.members)
        welcome_msg = (
            f"Welcome to hell, {member.mention}! We now number {current_member_count}!"
            f" Check out <#{CHANNELS['welcome-and-rules']}>"
            f" and <#{CHANNELS['intros']}> to get verified and"
            f" check out <id:customize> to get roles!"
        )
        await asyncio.gather(
            _text_channel(self, JOIN_LEAVE_MSG_CHANNEL).send(
                f"{member.mention} is number 413... the holy number..."
                if current_member_count == 413
                else welcome_msg
            ),
            member.send(welcome_msg),
            member.add_roles(
                *(
                    _member_role(member, role_name)
                    for role_name in DEFAULT_JOIN_ROLE_NAMES
                )
            ),
        )

    async def on_member_remove(self, member: discord.Member) -> None:
        await _text_channel(self, JOIN_LEAVE_MSG_CHANNEL).send(
            f"{_emoji('vriska')} {member.mention} couldn't bear the torture. "
            "Our population lowers to "
            f"{non_bot_member_count(member.guild.members)}. They'll be back."
        )


dave_bot = DaveBot(intents=discord.Intents.all())

EMOJIS: dict[str, EmojiResolver] = {
    "vriska": lambda: dave_bot.get_emoji(1017263376361062490),
    "thumbsupdirk": lambda: dave_bot.get_emoji(1016921360674598944),
    "johndab": lambda: dave_bot.get_emoji(1023722986332749834),
    "rosedab": lambda: dave_bot.get_emoji(1023722984680214528),
    "davedab": lambda: dave_bot.get_emoji(1023722989298122824),
    "jadedab": lambda: dave_bot.get_emoji(1023722987834331156),
}

EMOJI_NAMES_BY_FACTORY = {factory: name for name, factory in EMOJIS.items()}

DEFAULT_JOIN_ROLE_NAMES = (
    "color_divider",
    "location_divider",
    "ping_divider",
    "pronoun_divider",
    "classpect_divider",
    "misc_divider",
    "unverified",
)

EMOJI_TRIGGERS = {
    emoji: tuple(trigger.casefold() for trigger in triggers)
    for emoji, triggers in [
        (EMOJIS["vriska"], ("vriska", "serket")),
        (EMOJIS["johndab"], ("john", "egbert")),
        (EMOJIS["rosedab"], ("rose", "lalonde")),
        (EMOJIS["davedab"], ("dave", "strider")),
        (EMOJIS["jadedab"], ("jade", "harley")),
    ]
}


def _emoji_by_factory(
    factory: EmojiResolver,
) -> discord.Emoji | discord.PartialEmoji | discord.AppEmoji:
    try:
        return _emoji(EMOJI_NAMES_BY_FACTORY[factory])
    except KeyError as exc:
        raise RuntimeError("Emoji factory is not registered") from exc


def non_bot_member_count(members: Iterable[discord.Member]) -> int:
    return sum(not member.bot for member in members)


def _first_trigger_position(message: str, triggers: Iterable[str]) -> int | None:
    matches = [
        position for trigger in triggers if (position := message.find(trigger)) != -1
    ]
    return min(matches, default=None)


def _triggered_emojis(message: str) -> list[EmojiResolver]:
    casefolded_message = message.casefold()
    matches = [
        (emoji, position)
        for emoji, triggers in EMOJI_TRIGGERS.items()
        if (position := _first_trigger_position(casefolded_message, triggers))
        is not None
    ]
    return [emoji for emoji, _ in sorted(matches, key=lambda item: item[1])]


def _ordered_poll_emojis(
    message: str,
    custom_emojis: Iterable[discord.Emoji | discord.AppEmoji],
) -> list[PollEmoji]:
    emoji_positions: dict[PollEmoji, int] = {}

    for emoji in custom_emojis:
        if (position := message.find(str(emoji))) != -1:
            emoji_positions[emoji] = position

    for match in emoji_list(message):
        emoji_positions.setdefault(match["emoji"], match["match_start"])

    return [
        emoji for emoji, _ in sorted(emoji_positions.items(), key=lambda item: item[1])
    ]


async def _clear_bot_reactions(
    ctx: discord.ApplicationContext,
    message: discord.Message,
) -> None:
    if ctx.bot.user is None:
        raise RuntimeError("Bot user is not available")
    await asyncio.gather(
        *(reaction.remove(ctx.bot.user) for reaction in message.reactions)
    )


def _chunk_lines(lines: Iterable[str], limit: int = 2000) -> list[str]:
    chunks: list[str] = []
    pending_lines: list[str] = []
    pending_length = 0

    for line in lines:
        line_length = len(line)
        separator_length = 1 if pending_lines else 0
        if pending_lines and pending_length + separator_length + line_length > limit:
            chunks.append("\n".join(pending_lines))
            pending_lines = []
            pending_length = 0

        pending_lines.append(line)
        pending_length += line_length + (1 if len(pending_lines) > 1 else 0)

    if pending_lines:
        chunks.append("\n".join(pending_lines))

    return chunks


def _target_label(messageable: MessageTarget) -> str:
    if isinstance(messageable, discord.TextChannel | discord.User):
        return f"{messageable} ({messageable.mention})"
    return str(messageable)


def _sent_message_embed(
    message: discord.Message,
    *,
    author: EmbedAuthor,
    target: str,
) -> discord.Embed:
    embed = discord.Embed(
        title=f"To {target}",
        description=message.content,
        color=author.color if isinstance(author, discord.Member) else None,
        timestamp=message.created_at,
    )
    embed.add_field(name="Message ID", value=str(message.id))
    embed.add_field(name="Channel ID", value=str(message.channel.id))
    _set_embed_author(embed, author)
    return embed


staff_only = has_any_role(ROLES["mod"], ROLES["admin"])


async def msg(
    ctx: discord.ApplicationContext,
    messageable: MessageTarget,
    message: str,
) -> None:
    if ctx.channel_id != CHANNELS["davebot"]:
        await ctx.respond(
            f"U gotta run this command in <#{CHANNELS['davebot']}>",
            ephemeral=True,
        )
        return
    if messageable == dave_bot.user:
        await ctx.respond("Lmao why did u try to message me", ephemeral=True)
        return

    try:
        sent = await messageable.send(message)
    except discord.ApplicationCommandInvokeError as exc:
        await ctx.respond(f"Error:\n{exc}", ephemeral=True)
        raise

    embed = _sent_message_embed(
        sent,
        author=ctx.user,
        target=_target_label(messageable),
    )
    await asyncio.gather(
        _text_channel(dave_bot, CHANNELS["davebot"]).send(embed=embed),
        ctx.respond("Sent", ephemeral=True),
    )


message_cmds = dave_bot.create_group("message", "Sends messages as bot")


@message_cmds.command()
@staff_only
async def user(
    ctx: discord.ApplicationContext,
    user: discord.User,
    message: str,
) -> None:
    await msg(ctx, user, message)


@message_cmds.command()
@staff_only
async def channel(
    ctx: discord.ApplicationContext,
    channel: discord.TextChannel,
    message: str,
) -> None:
    await msg(ctx, channel, message)


class VerificationView(discord.ui.View):
    member: discord.Member

    def __init__(self, member: discord.Member) -> None:
        super().__init__()
        self.member = member

    @discord.ui.button(label="Verify without intro")
    async def button_callback(
        self,
        button: discord.ui.Button[Any],
        interaction: discord.Interaction,
    ) -> None:
        del button
        if interaction.guild is None:
            raise RuntimeError("Verification interaction must happen in a guild")
        if interaction.user is None:
            raise RuntimeError("Verification interaction must have a user")
        if _has_role(self.member, ROLES["member"]):
            await interaction.response.send_message(
                "User already verified",
                ephemeral=True,
            )
            return

        await asyncio.gather(
            self.member.add_roles(_role(interaction.guild, ROLES["member"])),
            self.member.remove_roles(_role(interaction.guild, ROLES["unverified"])),
        )
        await asyncio.gather(
            self.member.send(
                "Congratulations, you're now verified! Welcome to the server!"
            ),
            interaction.response.send_message(
                str(_emoji("thumbsupdirk")),
                ephemeral=True,
            ),
            _text_channel(dave_bot, CHANNELS["modlog"]).send(
                f"{interaction.user.mention} verified"
                f" {self.member.mention} without an intro",
            ),
        )


async def _verify(
    ctx: discord.ApplicationContext,
    member: discord.Member | discord.User,
    message: discord.Message | None = None,
) -> None:
    if ctx.guild is None:
        raise RuntimeError("Verification commands must run in a guild")
    if member.id == dave_bot.application_id:
        await ctx.respond("You can't verify me!", ephemeral=True)
        return
    if not isinstance(member, discord.Member):
        await ctx.respond("User no longer in the server", ephemeral=True)
        return
    if _has_role(member, ROLES["member"]):
        await ctx.respond("User already verified", ephemeral=True)
        return
    if message is None:
        await ctx.respond(
            "Are you sure you want to verify this user without an intro?",
            ephemeral=True,
            view=VerificationView(member),
        )
        return

    await asyncio.gather(
        member.add_roles(_role(ctx.guild, ROLES["member"])),
        member.remove_roles(_role(ctx.guild, ROLES["unverified"])),
    )
    await asyncio.gather(
        member.send("Congratulations, you're now verified! Welcome to the server!"),
        ctx.respond(str(_emoji("thumbsupdirk")), ephemeral=True),
        _text_channel(dave_bot, CHANNELS["modlog"]).send(
            f"{ctx.user.mention} verified {member.mention}",
            embed=_new_message_embed(message),
        ),
        message.add_reaction(_emoji("thumbsupdirk")),
    )


@dave_bot.user_command(name="Verify without intro", guild_ids=[GUILD])
@staff_only
async def user_verify(
    ctx: discord.ApplicationContext,
    user: discord.Member | discord.User,
) -> None:
    await _verify(ctx, user)


@dave_bot.message_command(name="Verify", guild_ids=[GUILD])
@staff_only
async def msg_verify(ctx: discord.ApplicationContext, message: discord.Message) -> None:
    await _verify(ctx, message.author, message)


@dave_bot.slash_command(name="list_unverified", description="Lists unverified members")
@staff_only
async def list_unverified(ctx: discord.ApplicationContext) -> None:
    if ctx.guild is None:
        raise RuntimeError("This command must run in a guild")

    unverified_members = sorted(
        (
            member
            for member in ctx.guild.members
            if not member.bot
            and member.joined_at is not None
            and not _has_role(member, ROLES["member"])
        ),
        key=lambda member: member.joined_at or datetime.min.replace(tzinfo=UTC),
    )
    entries = (
        f"{member.mention}: {format_dt(member.joined_at)}"
        for member in unverified_members
        if member.joined_at is not None
    )
    for chunk in _chunk_lines(entries):
        await ctx.respond(chunk, ephemeral=True)


@dave_bot.slash_command(
    name="member_count",
    description="The amount of non-bot members.",
)
async def member_count(ctx: discord.ApplicationContext) -> None:
    if ctx.guild is None:
        raise RuntimeError("This command must run in a guild")
    await ctx.respond(
        f"There are currently {non_bot_member_count(ctx.guild.members)}"
        f" non-bot members out of {len(ctx.guild.members)}"
        " in the server.",
        ephemeral=True,
    )


async def _add_reactions_in_order(
    message: discord.Message,
    emojis: Sequence[PollEmoji],
) -> None:
    for emoji in emojis:
        await message.add_reaction(emoji)


@dave_bot.message_command(name="Poll", guild_ids=[GUILD])
async def poll(ctx: discord.ApplicationContext, message: discord.Message) -> None:
    await ctx.respond("Removing reactions...", ephemeral=True)
    await _clear_bot_reactions(ctx, message)
    await ctx.respond("Reacting...", ephemeral=True)
    await _add_reactions_in_order(
        message,
        _ordered_poll_emojis(message.content, ctx.bot.emojis),
    )
    await ctx.respond(f"Done {_emoji('thumbsupdirk')}", ephemeral=True)


@dave_bot.message_command(name="Unreact", guild_ids=[GUILD])
async def unreact(ctx: discord.ApplicationContext, message: discord.Message) -> None:
    await ctx.respond("Removing reactions...", ephemeral=True)
    await _clear_bot_reactions(ctx, message)
    await ctx.respond(f"Done {_emoji('thumbsupdirk')}", ephemeral=True)


def run_bot(token: str) -> None:
    dave_bot.run(token)


def main() -> None:
    config = load_config()
    logging.basicConfig(level=logging.INFO)
    run_bot(config.token)


if __name__ == "__main__":
    main()
