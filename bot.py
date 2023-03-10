#!/usr/bin/env python3
import logging
import os
from itertools import chain
from typing import List

import discord
from discord.ext.commands import has_role
from emoji import emoji_list

GUILD = 980962249550213170

CHANNELS = {
    "general": 980962249550213176,
    "spam": 981995926883287142,
    "modlog": 981416669706608650,
}

ROLES = {
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

bot = discord.Bot(intents=discord.Intents.all())

EMOJIS = {
    "vriska": lambda: bot.get_emoji(1017263376361062490),
    "thumbsupdirk": lambda: bot.get_emoji(1016921360674598944),
    "johndab": lambda: bot.get_emoji(1023722986332749834),
    "rosedab": lambda: bot.get_emoji(1023722984680214528),
    "davedab": lambda: bot.get_emoji(1023722989298122824),
    "jadedab": lambda: bot.get_emoji(1023722987834331156),
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


@bot.listen()
async def on_message(message: discord.Message):
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


@bot.listen()
async def on_member_join(member: discord.Member):
    channel = bot.get_channel(JOIN_LEAVE_MSG_CHANNEL)
    await channel.send(
        f"Welcome to hell, {member.mention}! We now number {non_bot_member_count(member.guild.members)}!"
        " Check out <#980962249550213172> and <#980968056245354596> to get verified."
    )
    await member.add_roles(
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
    )


@bot.listen()
async def on_member_remove(member: discord.Member):
    channel = bot.get_channel(JOIN_LEAVE_MSG_CHANNEL)
    await channel.send(
        f"{EMOJIS['vriska']()} {member.mention} couldn't bear the torture. Our population lowers to "
        f"{non_bot_member_count(member.guild.members)}. They'll be back."
    )


async def _verify(ctx: discord.ApplicationContext, member: discord.Member):
    if member.id == bot.application_id:
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
    await member.send("Congratulations, you're now verified! Welcome to the server!")
    await ctx.respond(str(EMOJIS["thumbsupdirk"]()), ephemeral=True)
    await ctx.guild.get_channel(CHANNELS["modlog"]).send(
        f"{ctx.user.mention} verified {member.mention}"
    )


@bot.slash_command(name="verify", description="Mod command to verify new users.")
@has_role(ROLES["mod"])
async def verify(ctx: discord.ApplicationContext, member: discord.Member):
    await _verify(ctx, member)


@bot.user_command(name="Verify", guild_ids=[GUILD])
@has_role(ROLES["mod"])
async def user_verify(ctx: discord.ApplicationContext, member: discord.Member):
    await _verify(ctx, member)


@bot.message_command(name="Poll", guild_ids=[GUILD])
async def poll(ctx: discord.ApplicationContext, message: discord.Message):
    await ctx.respond("Removing reactions...", ephemeral=True)
    for reaction in message.reactions:
        await reaction.remove(ctx.bot.user)

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


@bot.message_command(name="Unreact", guild_ids=[GUILD])
async def unreact(ctx: discord.ApplicationContext, message: discord.Message):
    await ctx.respond("Removing reactions...", ephemeral=True)
    for reaction in message.reactions:
        await reaction.remove(ctx.bot.user)
    await ctx.respond(f"Done {EMOJIS['thumbsupdirk']()}", ephemeral=True)


def run_bot(token: str) -> None:
    bot.run(token)


def main() -> None:
    token = os.getenv("token")
    if not isinstance(token, str):
        raise TypeError("No Token")
    logging.basicConfig(level=logging.INFO)
    run_bot(token)


if __name__ == "__main__":
    main()
