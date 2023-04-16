#!/usr/bin/env python3
import asyncio
import logging
import os
from itertools import chain
from typing import List, Optional

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
}

JOIN_LEAVE_MSG_CHANNEL = CHANNELS["general"]


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
            await bot.get_guild(GUILD).get_channel(CHANNELS["davebot"]).send(
                embed=embed, files=files, stickers=message.stickers
            )
        except discord.errors.Forbidden:
            await bot.get_guild(GUILD).get_channel(CHANNELS["davebot"]).send(
                content=f"Message contained sticker which cannot be sent here.\nStickers: {message.stickers}",
                embed=embed,
                files=files,
            )


async def _on_message_react(bot: discord.Bot, message: discord.Message):
    if bot.application_id == message.author.id:
        return

    casefolded_message = message.content.casefold()
    emojis = (
        emoji_with_pos[0]
        for emoji_with_pos in sorted(
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
        if emoji_with_pos[1] != -1
    )
    if message.channel.id in [CHANNELS["spam"]]:
        reply = "".join(str(emoji()) for emoji in emojis)
        if len(reply) > 0:
            await message.reply(reply)
    else:
        for emoji in emojis:
            await message.add_reaction(emoji())


class DaveBot(discord.Bot):
    async def on_message(self, message: discord.Message):
        await asyncio.gather(
            _on_message_forward(self, message), _on_message_react(self, message)
        )

    async def on_member_join(self, member: discord.Member):
        welcome_msg = (
            f"Welcome to hell, {member.mention}! We now number {non_bot_member_count(member.guild.members)}!"
            f" Check out <#{CHANNELS['welcome-and-rules']}> and <#{CHANNELS['intros']}> to get verified and"
            f" check out <#{CHANNELS['roles']}> to get roles!"
        )
        await asyncio.gather(
            self.get_channel(JOIN_LEAVE_MSG_CHANNEL).send(welcome_msg),
            member.send(welcome_msg),
            member.add_roles(
                *(
                    member.guild.get_role(role)
                    for role in [
                        ROLES["color_divider"],
                        ROLES["location_divider"],
                        ROLES["ping_divider"],
                        ROLES["pronoun_divider"],
                        ROLES["classpect_divider"],
                        ROLES["misc_divider"],
                    ]
                )
            ),
        )

    async def on_member_remove(self, member: discord.Member):
        channel = self.get_channel(JOIN_LEAVE_MSG_CHANNEL)
        await channel.send(
            f"{EMOJIS['vriska']()} {member.mention} couldn't bear the torture. Our population lowers to "
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


def non_bot_member_count(members: List[discord.Member]) -> int:
    return sum(1 if not member.bot else 0 for member in members)


@dave_bot.slash_command(name="message", guild_ids=[GUILD])
@has_any_role(ROLES["mod"], ROLES["admin"])
async def dm(ctx: discord.ApplicationContext, user: discord.User, message: str):
    if ctx.channel_id != CHANNELS["davebot"]:
        await ctx.respond(
            f"U gotta run this command in <#{CHANNELS['davebot']}>", ephemeral=True
        )
        return
    if user == dave_bot.user:
        await ctx.respond(
            "Lmao why did u try to message me",
            ephemeral=True,
        )
        return
    try:
        sent = await user.send(message)
        embed = discord.Embed(
            title=f"To {user} ({user.mention})",
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
        await ctx.respond(embed=embed)
    except discord.ApplicationCommandInvokeError as e:
        await ctx.respond(f"Error:\n{e}", ephemeral=True)
        raise e


async def _verify(
    ctx: discord.ApplicationContext,
    member: discord.Member,
    message: Optional[discord.Message] = None,
):
    if member.id == dave_bot.application_id:
        await ctx.respond("You can't verify me!", ephemeral=True)
        return

    if member not in ctx.guild.members:
        await ctx.respond("User no longer in the server", ephemeral=True)
        return

    role = ctx.guild.get_role(ROLES["member"])
    if role in member.roles:
        await ctx.respond("User already verified", ephemeral=True)
        return
    await member.add_roles(role)
    coros = [
        member.send("Congratulations, you're now verified! Welcome to the server!"),
        ctx.respond(str(EMOJIS["thumbsupdirk"]()), ephemeral=True),
        ctx.guild.get_channel(CHANNELS["modlog"]).send(
            f"{ctx.user.mention} verified {member.mention}",
            embed=None if message is None else msg_embed(message),
        ),
    ]
    if message is not None:
        coros.append(message.add_reaction(EMOJIS["thumbsupdirk"]()))
    await asyncio.gather(*coros)


@dave_bot.user_command(name="Verify", guild_ids=[GUILD])
@has_any_role(ROLES["mod"], ROLES["admin"])
async def user_verify(ctx: discord.ApplicationContext, member: discord.Member):
    await _verify(ctx, member)


@dave_bot.message_command(name="Verify", guild_ids=[GUILD])
@has_any_role(ROLES["mod"], ROLES["admin"])
async def msg_verify(ctx: discord.ApplicationContext, message: discord.Message):
    if isinstance(message.author, discord.Member):
        await _verify(ctx, message.author, message)
    else:
        raise TypeError("User, not member")


@dave_bot.slash_command(name="list_unverified", description="Lists unverified members")
@has_any_role(ROLES["mod"], ROLES["admin"])
async def list_unverified(ctx: discord.ApplicationContext):
    entries = (
        f"{member.mention}: {format_dt(member.joined_at)}"
        for member in sorted(
            (
                member
                for member in ctx.guild.members
                if (not member.bot)
                and (ROLES["member"] not in (role.id for role in member.roles))
            ),
            key=lambda member: member.joined_at,
        )
    )
    queue = []
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
    if len(queue) > 0:
        await ctx.respond(
            "\n".join(queue),
            ephemeral=True,
        )


@dave_bot.slash_command(
    name="member_count", description="The amount of non-bot members."
)
async def member_count(ctx: discord.ApplicationContext):
    await ctx.respond(
        f"There are currently {non_bot_member_count(ctx.guild.members)} non-bot members out of {len(ctx.guild.members)}"
        " in the server.",
        ephemeral=True,
    )


@dave_bot.message_command(name="Poll", guild_ids=[GUILD])
async def poll(ctx: discord.ApplicationContext, message: discord.Message):
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
    await ctx.respond("Removing reactions...", ephemeral=True)
    await asyncio.gather(
        *(reaction.remove(ctx.bot.user) for reaction in message.reactions)
    )
    await ctx.respond(f"Done {EMOJIS['thumbsupdirk']()}", ephemeral=True)


def run_bot(token: str) -> None:
    dave_bot.run(token)


def main() -> None:
    token = os.getenv("token")
    if not isinstance(token, str):
        raise TypeError("No Token")
    logging.basicConfig(level=logging.INFO)
    run_bot(token)


if __name__ == "__main__":
    main()
