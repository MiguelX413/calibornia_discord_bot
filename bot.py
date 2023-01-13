#!/usr/bin/env python3
import logging
import os

import discord
from discord.ext.commands import has_role

GUILD = 980962249550213170

MOD_ROLE = 1027089314405957685

CHANNELS = {
    "general": 980962249550213176,
    "spam": 981995926883287142,
}

JOIN_LEAVE_MSG_CHANNEL = CHANNELS["general"]

MEMBER_COUNT = len([m for m in ctx.guild.members if not m.bot])

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
        (EMOJIS["vriska"], ["vriska"]),
        (EMOJIS["johndab"], ["john"]),
        (EMOJIS["rosedab"], ["rose"]),
        (EMOJIS["davedab"], ["dave"]),
        (EMOJIS["jadedab"], ["jade"]),
    ]
}


def emoji_text(emoji: discord.Emoji) -> str:
    return f"<:{emoji.name}:{emoji.id}>"


@bot.listen()
async def on_message(message: discord.Message):
    if bot.application_id == message.author.id:
        return
    casefolded_message = message.content.casefold()
    emojis = filter(
        lambda x: x is not None,
        (
            emoji
            if any(trigger in casefolded_message for trigger in EMOJI_TRIGGERS[emoji])
            else None
            for emoji in EMOJI_TRIGGERS
        ),
    )

    if message.channel.id in [CHANNELS["spam"]]:
        reply = "".join(emoji_text(emoji()) for emoji in emojis)
        if len(reply) > 0:
            await message.reply(reply)
    else:
        for emoji in emojis:
            await message.add_reaction(emoji())


@bot.listen()
async def on_member_join(member: discord.Member):
    channel = bot.get_channel(JOIN_LEAVE_MSG_CHANNEL)
    await channel.send(
        f"Welcome to hell, {member.mention}! We now number {MEMBER_COUNT}! Check out <#980968056245354596> to get verified."
    )


@bot.listen()
async def on_member_remove(member: discord.Member):
    channel = bot.get_channel(JOIN_LEAVE_MSG_CHANNEL)
    await channel.send(
        f"{EMOJIS['vriska']()} {member.mention} couldn't bear the torture. Our population lowers to {MEMBER_COUNT}. They'll be back."
    )


async def _verify(ctx: discord.ApplicationContext, member: discord.Member):
    if member.id == bot.application_id:
        await ctx.respond("You can't verify me!", ephemeral=True)
        return

    role = bot.get_guild(GUILD).get_role(982177726691700736)
    if role in member.roles:
        await ctx.respond("User already verified", ephemeral=True)
        return
    await member.add_roles(role)
    await member.send("Congratulations, you're now verified! Welcome to the server!")
    await ctx.respond(emoji_text(EMOJIS["thumbsupdirk"]()), ephemeral=True)


@bot.slash_command(name="verify", description="Mod command to verify new users.")
@has_role(MOD_ROLE)
async def verify(ctx: discord.ApplicationContext, member: discord.Member):
    await _verify(ctx, member)


@bot.user_command(name="Verify", guild_ids=[GUILD])
@has_role(MOD_ROLE)
async def user_verify(ctx: discord.ApplicationContext, member: discord.Member):
    await _verify(ctx, member)


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
