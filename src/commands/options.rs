use anyhow::{Result, anyhow};
use serenity::all::{ResolvedOption, ResolvedValue, User};

pub(super) fn string_option<'a>(options: &'a [ResolvedOption<'a>], name: &str) -> Result<&'a str> {
    options
        .iter()
        .find(|option| option.name == name)
        .and_then(|option| match option.value {
            ResolvedValue::String(value) => Some(value),
            _ => None,
        })
        .ok_or_else(|| anyhow!("missing string option {name}"))
}

pub(super) fn user_option<'a>(options: &'a [ResolvedOption<'a>], name: &str) -> Result<&'a User> {
    options
        .iter()
        .find(|option| option.name == name)
        .and_then(|option| match option.value {
            ResolvedValue::User(user, _) => Some(user),
            _ => None,
        })
        .ok_or_else(|| anyhow!("missing user option {name}"))
}

pub(super) fn channel_option<'a>(
    options: &'a [ResolvedOption<'a>],
    name: &str,
) -> Result<&'a serenity::all::PartialChannel> {
    options
        .iter()
        .find(|option| option.name == name)
        .and_then(|option| match option.value {
            ResolvedValue::Channel(channel) => Some(channel),
            _ => None,
        })
        .ok_or_else(|| anyhow!("missing channel option {name}"))
}
