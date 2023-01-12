#!/usr/bin/env python3
import logging
import os
from typing import Optional

import discord

bot = discord.Bot()


@bot.slash_command()
async def hello(ctx: discord.ApplicationContext, name: Optional[str] = None):
    name = name or ctx.author.name
    await ctx.respond(f"Hello {name}!")


@bot.slash_command()
async def hi(ctx: discord.ApplicationContext, user: discord.User):
    await ctx.respond(f"{ctx.author.mention} says hello to {user.name}!")


@bot.listen()
async def on_message(message: discord.Message):
    if bot.application_id == message.author.id:
        return
    if "vriska".casefold() in message.content.casefold():
        await message.reply("<:vriska:1017263376361062490>")


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
