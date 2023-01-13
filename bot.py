#!/usr/bin/env python3
import logging
import os
from typing import Optional

import discord

bot = discord.Bot()

@bot.listen()
async def on_message(message: discord.Message):
    if bot.application_id == message.author.id:
        return
    if "vriska".casefold() in message.content.casefold():
        await message.reply("<:vriska:1017263376361062490>")
          
@bot.event
async def on_member_join(member):
    channel = client.get_channel(980962249550213176)
    await channel.send(
        f'Welcome to the server, {member.mention}! Enjoy your stay here.'
    )

@bot.slash_command(name ="verify", description="Mod command to verify new users.")
async def verify (ctx, member: discord.Member):
    role = discord.utils.get(ctx.guild.roles, name="member")
    await member.add_roles(role)
    await member.send(
        f"Congratulations, you're now verified! Welcome to the server!"
    )
    await ctx.respond(f"<:thumbsupdirk:1016921360674598944>")        


def run_bot(token: str) -> None:
    bot.run(token)


def main() -> None:
    token = os.environ.get("token", None)
    if not isinstance(token, str):
        raise TypeError("No Token")
    logging.basicConfig(level=logging.INFO)
    run_bot(token)


if __name__ == "__main__":
    main()
