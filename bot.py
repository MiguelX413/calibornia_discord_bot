#!/usr/bin/env python3
import asyncio
import logging
import os
from datetime import UTC, datetime
from itertools import chain
from typing import Any

import discord
from discord.ext.commands import has_any_role
from discord.utils import format_dt
from emoji import emoji_list

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


def msg_embed(message: discord.Message) -> discord.Embed:
    fields = [
        discord.EmbedField("Message ID", f"{message.id}"),
        discord.EmbedField("Channel ID", f"{message.channel.id}"),
    ]
    if message.reference is not None:
        fields.append(
            discord.EmbedField("Reference", f"{message.reference.message_id}")
        )
    embed = discord.Embed(
        description=message.content,
        color=message.author.color,
        timestamp=message.created_at,
        fields=fields,
    )
    embed.set_author(
        name=f"{message.author} ({message.author.mention})",
        url=f"https://discordapp.com/users/{message.author.id}",
        icon_url=message.author.avatar,
    )
    return embed


async def _on_message_forward(bot: discord.Bot, message: discord.Message):
    if bot.application_id == message.author.id:
        return

    if message.guild is None:
        embed = msg_embed(message)
        files = [
            (await attachment.to_file(use_cached=True))
            for attachment in message.attachments
        ]
        try:
            await _text_channel(bot, CHANNELS["davebot"]).send(
                embed=embed, files=files, stickers=message.stickers
            )
        except discord.errors.Forbidden:
            await _text_channel(bot, CHANNELS["davebot"]).send(
                content=(
                    "Message contained sticker which cannot be sent here.\n"
                    f"Stickers: {message.stickers}"
                ),
                embed=embed,
                files=files,
            )


async def _on_message_react(bot: discord.Bot, message: discord.Message):
    if bot.application_id == message.author.id:
        return

    casefolded_message = message.content.casefold()
    emojis = (
        emoji
        for emoji, pos in sorted(
            (
                (
                    potential_emoji,
                    min(
                        (
                            pos
                            for pos in (
                                casefolded_message.find(trigger)
                                for trigger in EMOJI_TRIGGERS[potential_emoji]
                            )
                            if pos != -1
                        ),
                        default=-1,
                    ),
                )
                for potential_emoji in EMOJI_TRIGGERS
            ),
            key=lambda x: x[1],
        )
        if pos != -1
    )
    if message.channel.id == CHANNELS["spam"]:
        reply = "".join(str(emoji()) for emoji in emojis)
        if reply:
            await message.reply(reply)
    else:
        for emoji in emojis:
            await message.add_reaction(_emoji_by_factory(emoji))


def _emoji_by_factory(
    factory: Any,
) -> discord.Emoji | discord.PartialEmoji | discord.AppEmoji:
    for name, candidate in EMOJIS.items():
        if candidate is factory:
            return _emoji(name)
    raise RuntimeError("Emoji factory is not registered")


class DaveBot(discord.Bot):
    async def on_message(self, message: discord.Message):
        await asyncio.gather(
            _on_message_forward(self, message), _on_message_react(self, message)
        )

    async def on_member_join(self, member: discord.Member):
        _non_bot_member_count = non_bot_member_count(member.guild.members)
        welcome_msg = (
            f"Welcome to hell, {member.mention}! We now number {_non_bot_member_count}!"
            f" Check out <#{CHANNELS['welcome-and-rules']}>"
            f" and <#{CHANNELS['intros']}> to get verified and"
            f" check out <id:customize> to get roles!"
        )
        await asyncio.gather(
            _text_channel(self, JOIN_LEAVE_MSG_CHANNEL).send(
                f"{member.mention} is number 413... the holy number..."
                if _non_bot_member_count == 413
                else welcome_msg
            ),
            member.send(welcome_msg),
            member.add_roles(
                *(
                    _role(member.guild, role)
                    for role in [
                        ROLES["color_divider"],
                        ROLES["location_divider"],
                        ROLES["ping_divider"],
                        ROLES["pronoun_divider"],
                        ROLES["classpect_divider"],
                        ROLES["misc_divider"],
                        ROLES["unverified"],
                    ]
                )
            ),
        )

    async def on_member_remove(self, member: discord.Member):
        channel = _text_channel(self, JOIN_LEAVE_MSG_CHANNEL)
        await channel.send(
            f"{_emoji('vriska')} {member.mention} couldn't bear the torture. "
            "Our population lowers to "
            f"{non_bot_member_count(member.guild.members)}. They'll be back."
        )


dave_bot = DaveBot(intents=discord.Intents.all())

EMOJIS = {
    "vriska": lambda: dave_bot.get_emoji(1017263376361062490),
    "thumbsupdirk": lambda: dave_bot.get_emoji(1016921360674598944),
    "johndab": lambda: dave_bot.get_emoji(1023722986332749834),
    "rosedab": lambda: dave_bot.get_emoji(1023722984680214528),
    "davedab": lambda: dave_bot.get_emoji(1023722989298122824),
    "jadedab": lambda: dave_bot.get_emoji(1023722987834331156),
}

EMOJI_TRIGGERS = {
    emoji: list(trigger.casefold() for trigger in triggers)
    for emoji, triggers in [
        (EMOJIS["vriska"], ["vriska", "serket"]),
        (EMOJIS["johndab"], ["john", "egbert"]),
        (EMOJIS["rosedab"], ["rose", "lalonde"]),
        (EMOJIS["davedab"], ["dave", "strider"]),
        (EMOJIS["jadedab"], ["jade", "harley"]),
    ]
}


def non_bot_member_count(members: list[discord.Member]) -> int:
    return sum(not member.bot for member in members)


staff_only = has_any_role(ROLES["mod"], ROLES["admin"])


async def msg(
    ctx: discord.ApplicationContext, messageable: MessageTarget, message: str
):
    if ctx.channel_id != CHANNELS["davebot"]:
        await ctx.respond(
            f"U gotta run this command in <#{CHANNELS['davebot']}>", ephemeral=True
        )
        return
    if messageable == dave_bot.user:
        await ctx.respond(
            "Lmao why did u try to message me",
            ephemeral=True,
        )
        return
    try:
        sent = await messageable.send(message)
        embed = discord.Embed(
            title=(
                f"To {messageable} ({messageable.mention})"
                if isinstance(messageable, (discord.TextChannel, discord.User))
                else f"To {messageable}"
            ),
            description=message,
            color=ctx.user.color,
            fields=[
                discord.EmbedField("Message ID", f"{sent.id}"),
                discord.EmbedField("Channel ID", f"{sent.channel.id}"),
            ],
            timestamp=sent.created_at,
        )
        embed.set_author(
            name=f"{ctx.user} ({ctx.user.mention})",
            url=f"https://discordapp.com/users/{ctx.user.id}",
            icon_url=ctx.user.avatar,
        )
        await asyncio.gather(
            _text_channel(dave_bot, CHANNELS["davebot"]).send(embed=embed),
            ctx.respond("Sent", ephemeral=True),
        )
    except discord.ApplicationCommandInvokeError as e:
        await ctx.respond(f"Error:\n{e}", ephemeral=True)
        raise


message_cmds = dave_bot.create_group("message", "Sends messages as bot")


@message_cmds.command()
@staff_only
async def user(ctx: discord.ApplicationContext, user: discord.User, message: str):
    await msg(ctx, user, message)


@message_cmds.command()
@staff_only
async def channel(
    ctx: discord.ApplicationContext, channel: discord.TextChannel, message: str
):
    await msg(ctx, channel, message)


class VerificationView(discord.ui.View):
    member: discord.Member

    def __init__(self, member: discord.Member) -> None:
        super().__init__()
        self.member = member

    @discord.ui.button(label="Verify without intro")
    async def button_callback(
        self, button: discord.ui.Button[Any], interaction: discord.Interaction
    ):
        del button
        if interaction.guild is None:
            raise RuntimeError("Verification interaction must happen in a guild")
        if interaction.user is None:
            raise RuntimeError("Verification interaction must have a user")
        if ROLES["member"] in (role.id for role in self.member.roles):
            await interaction.response.send_message(
                "User already verified", ephemeral=True
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
                str(_emoji("thumbsupdirk")), ephemeral=True
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
):
    if ctx.guild is None:
        raise RuntimeError("Verification commands must run in a guild")
    if member.id == dave_bot.application_id:
        await ctx.respond("You can't verify me!", ephemeral=True)
        return

    if not isinstance(member, discord.Member):
        await ctx.respond("User no longer in the server", ephemeral=True)
        return

    if ROLES["member"] in (role.id for role in member.roles):
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
            embed=msg_embed(message),
        ),
        message.add_reaction(_emoji("thumbsupdirk")),
    )


@dave_bot.user_command(name="Verify without intro", guild_ids=[GUILD])
@staff_only
async def user_verify(
    ctx: discord.ApplicationContext, user: discord.Member | discord.User
):
    await _verify(ctx, user)


@dave_bot.message_command(name="Verify", guild_ids=[GUILD])
@staff_only
async def msg_verify(ctx: discord.ApplicationContext, message: discord.Message):
    await _verify(ctx, message.author, message)


@dave_bot.slash_command(name="list_unverified", description="Lists unverified members")
@staff_only
async def list_unverified(ctx: discord.ApplicationContext):
    if ctx.guild is None:
        raise RuntimeError("This command must run in a guild")
    unverified_members = sorted(
        (
            member
            for member in ctx.guild.members
            if (not member.bot)
            and member.joined_at is not None
            and (ROLES["member"] not in (role.id for role in member.roles))
        ),
        key=lambda member: member.joined_at or datetime.min.replace(tzinfo=UTC),
    )
    entries = (
        f"{member.mention}: {format_dt(member.joined_at)}"
        for member in unverified_members
        if member.joined_at is not None
    )
    queue: list[str] = []
    queue_total_len = 0
    for entry in entries:
        if queue_total_len + len(entry) + len(queue) > 2000:
            await ctx.respond(
                "\n".join(queue),
                ephemeral=True,
            )
            queue = []
            queue_total_len = 0
        queue.append(entry)
        queue_total_len += len(entry)
    if queue:
        await ctx.respond(
            "\n".join(queue),
            ephemeral=True,
        )


@dave_bot.slash_command(
    name="member_count", description="The amount of non-bot members."
)
async def member_count(ctx: discord.ApplicationContext):
    if ctx.guild is None:
        raise RuntimeError("This command must run in a guild")
    await ctx.respond(
        f"There are currently {non_bot_member_count(ctx.guild.members)}"
        f" non-bot members out of {len(ctx.guild.members)}"
        " in the server.",
        ephemeral=True,
    )


@dave_bot.message_command(name="Poll", guild_ids=[GUILD])
async def poll(ctx: discord.ApplicationContext, message: discord.Message):
    if ctx.bot.user is None:
        raise RuntimeError("Bot user is not available")
    await ctx.respond("Removing reactions...", ephemeral=True)
    await asyncio.gather(
        *(reaction.remove(ctx.bot.user) for reaction in message.reactions)
    )

    await ctx.respond("Reacting...", ephemeral=True)
    for emoji in (
        emoji
        for emoji, pos in sorted(
            chain(
                ((emoji, message.content.find(str(emoji))) for emoji in ctx.bot.emojis),
                (
                    (match["emoji"], match["match_start"])
                    for match in emoji_list(message.content)
                ),
            ),
            key=lambda x: x[1],
        )
        if pos != -1
    ):
        await message.add_reaction(emoji)

    await ctx.respond(f"Done {EMOJIS['thumbsupdirk']()}", ephemeral=True)


@dave_bot.message_command(name="Unreact", guild_ids=[GUILD])
async def unreact(ctx: discord.ApplicationContext, message: discord.Message):
    if ctx.bot.user is None:
        raise RuntimeError("Bot user is not available")
    await ctx.respond("Removing reactions...", ephemeral=True)
    await asyncio.gather(
        *(reaction.remove(ctx.bot.user) for reaction in message.reactions)
    )
    await ctx.respond(f"Done {EMOJIS['thumbsupdirk']()}", ephemeral=True)


def run_bot(token: str) -> None:
    dave_bot.run(token)


def main() -> None:
    token = os.getenv("TOKEN") or os.getenv("token")
    if not isinstance(token, str):
        raise TypeError("No Token")
    logging.basicConfig(level=logging.INFO)
    run_bot(token)


if __name__ == "__main__":
    main()
