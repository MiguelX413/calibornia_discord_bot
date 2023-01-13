#!/usr/bin/env python3
import logging
import os

import discord
from discord.ext.commands import has_role

GUILD = 980962249550213170

MOD_ROLE = 1027089314405957685

bot = discord.Bot(intents=discord.Intents.all())

EMOJIS = {
    "vriska": lambda: bot.get_emoji(1017263376361062490),
    "thumbsupdirk": lambda: bot.get_emoji(1016921360674598944),
}

EMOJI_TRIGGERS = {
    emoji: list(trigger.casefold() for trigger in triggers)
    for emoji, triggers in [(EMOJIS["vriska"], ["vriska"])]
}


def emoji_text(emoji: discord.Emoji) -> str:
    return f"<:{emoji.name}:{emoji.id}>"


@bot.listen()
async def on_message(message: discord.Message):
    if bot.application_id == message.author.id:
        return
    casefolded_message = message.content.casefold()
    emojis = list(
        filter(
            lambda x: x is not None,
            (
                emoji
                if any(
                    (trigger in casefolded_message) for trigger in EMOJI_TRIGGERS[emoji]
                )
                else None
                for emoji in EMOJI_TRIGGERS
            ),
        )
    )
    if message.channel.id in [981995926883287142]:
        if len(emojis) > 0:
            await message.reply("".join(emoji_text(emoji()) for emoji in emojis))
    else:
        for emoji in emojis:
            await message.add_reaction(emoji())


@bot.event
async def on_member_join(member: discord.Member):
    channel = bot.get_channel(980962249550213176)
    await channel.send(
        f"Welcome to the server, {member.mention}! Enjoy your stay here."
    )


async def _verify(ctx: discord.ApplicationContext, member: discord.Member):
    if member.id == bot.application_id:
        await ctx.respond("You can't verify me!")
        return

    role = bot.get_guild(GUILD).get_role(982177726691700736)
    await member.add_roles(role)
    await member.send("Congratulations, you're now verified! Welcome to the server!")
    await ctx.respond(emoji_text(EMOJIS["thumbsupdirk"]()))


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
