use serenity::all::{Context, CreateEmbed, CreateEmbedAuthor, Mentionable, Message, User};

pub(crate) fn new_message_embed(ctx: &Context, message: &Message) -> CreateEmbed {
    let mut embed = CreateEmbed::new()
        .description(message.content.clone())
        .timestamp(message.timestamp)
        .field("Message ID", message.id.to_string(), false)
        .field("Channel ID", message.channel_id.to_string(), false)
        .author(embed_author(&message.author));

    if let Some(reference) = &message.message_reference
        && let Some(message_id) = reference.message_id
    {
        embed = embed.field("Reference", message_id.to_string(), false);
    }

    if let Some(color) = message
        .member
        .as_ref()
        .and_then(|member| member_color(ctx, member))
    {
        embed = embed.color(color);
    }

    embed
}

fn member_color(
    ctx: &Context,
    member: &serenity::all::PartialMember,
) -> Option<serenity::all::Colour> {
    let guild_id = member.guild_id?;
    let guild = guild_id.to_guild_cached(&ctx.cache)?;
    member
        .roles
        .iter()
        .filter_map(|role_id| guild.roles.get(role_id))
        .filter(|role| role.colour.0 != 0)
        .max_by_key(|role| role.position)
        .map(|role| role.colour)
}

pub(crate) fn sent_message_embed(message: &Message, author: &User, target: &str) -> CreateEmbed {
    CreateEmbed::new()
        .title(format!("To {target}"))
        .description(message.content.clone())
        .timestamp(message.timestamp)
        .field("Message ID", message.id.to_string(), false)
        .field("Channel ID", message.channel_id.to_string(), false)
        .author(embed_author(author))
}

fn embed_author(user: &User) -> CreateEmbedAuthor {
    CreateEmbedAuthor::new(format!("{} ({})", user.tag(), user.mention()))
        .url(format!("https://discordapp.com/users/{}", user.id.get()))
        .icon_url(user.face())
}

pub(crate) fn target_label_user(user: &User) -> String {
    format!("{} ({})", user.tag(), user.mention())
}
