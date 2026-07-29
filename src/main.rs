use std::{collections::HashMap, env, sync::OnceLock};

use anyhow::{Context as _, Result, anyhow, ensure};
use regex::Regex;
use serenity::{
    Client,
    all::{
        ButtonStyle, ChannelId, CommandInteraction, CommandOptionType, CommandType,
        ComponentInteraction, Context, CreateActionRow, CreateAttachment, CreateButton,
        CreateCommand, CreateCommandOption, CreateEmbed, CreateEmbedAuthor,
        CreateInteractionResponse, CreateInteractionResponseFollowup,
        CreateInteractionResponseMessage, CreateMessage, EditInteractionResponse, EventHandler,
        GatewayIntents, GuildId, Interaction, Member, Mentionable, Message, ReactionType, Ready,
        ResolvedTarget, ResolvedValue, RoleId, User, UserId, async_trait,
    },
};
use tracing::{error, info};

const GUILD: GuildId = GuildId::new(980_962_249_550_213_170);

const INTROS: ChannelId = ChannelId::new(980_968_056_245_354_596);
const WELCOME_AND_RULES: ChannelId = ChannelId::new(980_962_249_550_213_172);
const GENERAL: ChannelId = ChannelId::new(980_962_249_550_213_176);
const SPAM: ChannelId = ChannelId::new(981_995_926_883_287_142);
const MODLOG: ChannelId = ChannelId::new(981_416_669_706_608_650);
const DAVEBOT: ChannelId = ChannelId::new(1_089_751_694_352_584_725);

const ADMIN: RoleId = RoleId::new(980_964_927_164_518_470);
const MEMBER: RoleId = RoleId::new(982_177_726_691_700_736);
const MOD: RoleId = RoleId::new(1_027_089_314_405_957_685);
const COLOR_DIVIDER: RoleId = RoleId::new(1_027_311_103_014_862_888);
const LOCATION_DIVIDER: RoleId = RoleId::new(1_027_310_335_314_628_708);
const PING_DIVIDER: RoleId = RoleId::new(1_027_095_201_262_616_607);
const PRONOUN_DIVIDER: RoleId = RoleId::new(1_027_094_772_848_005_160);
const CLASSPECT_DIVIDER: RoleId = RoleId::new(1_027_309_033_373_310_987);
const MISC_DIVIDER: RoleId = RoleId::new(1_027_309_906_807_750_676);
const UNVERIFIED: RoleId = RoleId::new(1_098_091_859_743_612_948);

const VRISKA: u64 = 1_017_263_376_361_062_490;
const THUMBSUPDIRK: u64 = 1_016_921_360_674_598_944;
const JOHNDAB: u64 = 1_023_722_986_332_749_834;
const ROSEDAB: u64 = 1_023_722_984_680_214_528;
const DAVEDAB: u64 = 1_023_722_989_298_122_824;
const JADEDAB: u64 = 1_023_722_987_834_331_156;

const DEFAULT_JOIN_ROLES: &[RoleId] = &[
    COLOR_DIVIDER,
    LOCATION_DIVIDER,
    PING_DIVIDER,
    PRONOUN_DIVIDER,
    CLASSPECT_DIVIDER,
    MISC_DIVIDER,
    UNVERIFIED,
];

#[derive(Clone, Copy)]
struct CustomEmoji {
    name: &'static str,
    id: u64,
}

impl CustomEmoji {
    fn reaction(self) -> ReactionType {
        ReactionType::Custom {
            animated: false,
            id: self.id.into(),
            name: Some(self.name.to_owned()),
        }
    }

    fn mention(self) -> String {
        format!("<:{}:{}>", self.name, self.id)
    }
}

const EMOJI_TRIGGERS: &[(CustomEmoji, &[&str])] = &[
    (
        CustomEmoji {
            name: "vriska",
            id: VRISKA,
        },
        &["vriska", "serket"],
    ),
    (
        CustomEmoji {
            name: "johndab",
            id: JOHNDAB,
        },
        &["john", "egbert"],
    ),
    (
        CustomEmoji {
            name: "rosedab",
            id: ROSEDAB,
        },
        &["rose", "lalonde"],
    ),
    (
        CustomEmoji {
            name: "davedab",
            id: DAVEDAB,
        },
        &["dave", "strider"],
    ),
    (
        CustomEmoji {
            name: "jadedab",
            id: JADEDAB,
        },
        &["jade", "harley"],
    ),
];

struct Config {
    discord_token: String,
}

impl Config {
    fn from_env() -> Result<Self> {
        let discord_token = env::var("DISCORD_TOKEN")
            .or_else(|_| env::var("TOKEN"))
            .context("missing Discord token; set DISCORD_TOKEN or TOKEN")?;
        ensure!(
            !discord_token.trim().is_empty(),
            "Discord token environment variable must not be empty"
        );

        Ok(Self { discord_token })
    }
}

fn thumbsupdirk() -> CustomEmoji {
    CustomEmoji {
        name: "thumbsupdirk",
        id: THUMBSUPDIRK,
    }
}

fn vriska() -> CustomEmoji {
    CustomEmoji {
        name: "vriska",
        id: VRISKA,
    }
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("connected as {}", ready.user.tag());

        if let Err(err) = register_commands(&ctx).await {
            error!(?err, "failed to register guild commands");
        }
    }

    async fn message(&self, ctx: Context, message: Message) {
        let forward = forward_dm(&ctx, &message);
        let react = react_to_triggers(&ctx, &message);

        let (forward_result, react_result) = tokio::join!(forward, react);
        if let Err(err) = forward_result {
            error!(?err, "failed to forward message");
        }
        if let Err(err) = react_result {
            error!(?err, "failed to react to message");
        }
    }

    async fn guild_member_addition(&self, ctx: Context, member: Member) {
        if let Err(err) = member_joined(&ctx, member).await {
            error!(?err, "failed to handle member join");
        }
    }

    async fn guild_member_removal(
        &self,
        ctx: Context,
        _guild_id: GuildId,
        user: User,
        member_data_if_available: Option<Member>,
    ) {
        if let Err(err) = member_left(&ctx, user, member_data_if_available).await {
            error!(?err, "failed to handle member removal");
        }
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        let result = match interaction {
            Interaction::Command(command) => handle_command(&ctx, command).await,
            Interaction::Component(component) => handle_component(&ctx, component).await,
            _ => Ok(()),
        };

        if let Err(err) = result {
            error!(?err, "failed to handle interaction");
        }
    }
}

async fn register_commands(ctx: &Context) -> Result<()> {
    GUILD
        .set_commands(
            &ctx.http,
            vec![
                CreateCommand::new("message")
                    .description("Sends messages as bot")
                    .add_option(
                        CreateCommandOption::new(
                            CommandOptionType::SubCommand,
                            "user",
                            "Sends a DM as the bot",
                        )
                        .add_sub_option(
                            CreateCommandOption::new(
                                CommandOptionType::User,
                                "user",
                                "Target user",
                            )
                            .required(true),
                        )
                        .add_sub_option(
                            CreateCommandOption::new(
                                CommandOptionType::String,
                                "message",
                                "Message",
                            )
                            .required(true),
                        ),
                    )
                    .add_option(
                        CreateCommandOption::new(
                            CommandOptionType::SubCommand,
                            "channel",
                            "Sends a channel message as the bot",
                        )
                        .add_sub_option(
                            CreateCommandOption::new(
                                CommandOptionType::Channel,
                                "channel",
                                "Target channel",
                            )
                            .required(true),
                        )
                        .add_sub_option(
                            CreateCommandOption::new(
                                CommandOptionType::String,
                                "message",
                                "Message",
                            )
                            .required(true),
                        ),
                    ),
                CreateCommand::new("Verify without intro").kind(CommandType::User),
                CreateCommand::new("Verify").kind(CommandType::Message),
                CreateCommand::new("list_unverified").description("Lists unverified members"),
                CreateCommand::new("member_count").description("The amount of non-bot members."),
                CreateCommand::new("Poll").kind(CommandType::Message),
                CreateCommand::new("Unreact").kind(CommandType::Message),
            ],
        )
        .await
        .context("registering commands")?;
    Ok(())
}

async fn forward_dm(ctx: &Context, message: &Message) -> Result<()> {
    if message.author.bot || message.guild_id.is_some() {
        return Ok(());
    }

    let mut files = Vec::with_capacity(message.attachments.len());
    for attachment in &message.attachments {
        let bytes = attachment
            .download()
            .await
            .with_context(|| format!("downloading attachment {}", attachment.id))?;
        files.push(CreateAttachment::bytes(bytes, attachment.filename.clone()));
    }

    let embed = new_message_embed(ctx, message);
    let sticker_ids: Vec<_> = message
        .sticker_items
        .iter()
        .map(|sticker| sticker.id)
        .collect();
    let builder = CreateMessage::new()
        .embed(embed.clone())
        .files(files.clone())
        .sticker_ids(sticker_ids);

    if DAVEBOT.send_message(&ctx.http, builder).await.is_err() {
        let sticker_list = format!("{:?}", message.sticker_items);
        DAVEBOT
            .send_message(
                &ctx.http,
                CreateMessage::new()
                    .content(format!(
                        "Message contained sticker which cannot be sent here.\nStickers: {sticker_list}"
                    ))
                    .embed(embed)
                    .files(files),
            )
            .await
            .context("forwarding DM without stickers")?;
    }

    Ok(())
}

async fn react_to_triggers(ctx: &Context, message: &Message) -> Result<()> {
    if message.author.bot {
        return Ok(());
    }

    let emojis = triggered_emojis(&message.content);
    if message.channel_id == SPAM {
        let reply = emojis
            .iter()
            .map(|emoji| emoji.mention())
            .collect::<String>();
        if !reply.is_empty() {
            message
                .reply(&ctx.http, reply)
                .await
                .context("replying with emojis")?;
        }
        return Ok(());
    }

    for emoji in emojis {
        message
            .react(&ctx.http, emoji.reaction())
            .await
            .with_context(|| format!("reacting with {}", emoji.name))?;
    }

    Ok(())
}

async fn member_joined(ctx: &Context, member: Member) -> Result<()> {
    let member_count = non_bot_member_count(ctx);
    let welcome_msg = format!(
        "Welcome to hell, {}! We now number {member_count}! Check out <#{}> and <#{}> to get verified and check out <id:customize> to get roles!",
        member.mention(),
        WELCOME_AND_RULES.get(),
        INTROS.get(),
    );
    let channel_msg = if member_count == 413 {
        format!("{} is number 413... the holy number...", member.mention())
    } else {
        welcome_msg.clone()
    };

    let send_channel = GENERAL.say(&ctx.http, channel_msg);
    let send_dm = member
        .user
        .direct_message(&ctx.http, CreateMessage::new().content(welcome_msg));
    let add_roles = member.add_roles(&ctx.http, DEFAULT_JOIN_ROLES);

    let (channel_result, dm_result, roles_result) = tokio::join!(send_channel, send_dm, add_roles);
    channel_result.context("sending welcome message")?;
    dm_result.context("sending welcome DM")?;
    roles_result.context("adding default roles")?;

    Ok(())
}

async fn member_left(
    ctx: &Context,
    user: User,
    member_data_if_available: Option<Member>,
) -> Result<()> {
    let count = member_data_if_available
        .as_ref()
        .map(|_| non_bot_member_count(ctx))
        .unwrap_or_else(|| non_bot_member_count(ctx));

    GENERAL
        .say(
            &ctx.http,
            format!(
                "{} {} couldn't bear the torture. Our population lowers to {count}. They'll be back.",
                vriska().mention(),
                user.mention()
            ),
        )
        .await
        .context("sending leave message")?;
    Ok(())
}

async fn handle_command(ctx: &Context, command: CommandInteraction) -> Result<()> {
    match command.data.name.as_str() {
        "message" => {
            ensure_staff(ctx, &command).await?;
            handle_message_command(ctx, &command).await
        }
        "Verify without intro" => {
            ensure_staff(ctx, &command).await?;
            handle_user_verify(ctx, &command).await
        }
        "Verify" => {
            ensure_staff(ctx, &command).await?;
            handle_message_verify(ctx, &command).await
        }
        "list_unverified" => {
            ensure_staff(ctx, &command).await?;
            handle_list_unverified(ctx, &command).await
        }
        "member_count" => handle_member_count(ctx, &command).await,
        "Poll" => handle_poll(ctx, &command).await,
        "Unreact" => handle_unreact(ctx, &command).await,
        _ => Ok(()),
    }
}

async fn ensure_staff(ctx: &Context, command: &CommandInteraction) -> Result<()> {
    let Some(guild_id) = command.guild_id else {
        respond_ephemeral(ctx, command, "This command must run in a guild").await?;
        return Err(anyhow!("staff command used outside guild"));
    };

    let member = match &command.member {
        Some(member) => member,
        None => {
            respond_ephemeral(ctx, command, "This command must run in a guild").await?;
            return Err(anyhow!("missing command member"));
        }
    };

    let is_staff = member
        .roles
        .iter()
        .any(|role| *role == MOD || *role == ADMIN);
    if !is_staff {
        respond_ephemeral(
            ctx,
            command,
            "You do not have permission to use this command",
        )
        .await?;
        return Err(anyhow!(
            "non-staff user {} attempted command in {}",
            command.user.id,
            guild_id
        ));
    }

    Ok(())
}

async fn handle_message_command(ctx: &Context, command: &CommandInteraction) -> Result<()> {
    if command.channel_id != DAVEBOT {
        respond_ephemeral(
            ctx,
            command,
            &format!("U gotta run this command in <#{}>", DAVEBOT.get()),
        )
        .await?;
        return Ok(());
    }

    let subcommand = command
        .data
        .options()
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("missing message subcommand"))?;

    let ResolvedValue::SubCommand(options) = subcommand.value else {
        return Err(anyhow!("message command option was not a subcommand"));
    };

    let message_text = string_option(&options, "message")?;
    match subcommand.name {
        "user" => {
            let user = user_option(&options, "user")?;
            if user.id == ctx.cache.current_user().id {
                respond_ephemeral(ctx, command, "Lmao why did u try to message me").await?;
                return Ok(());
            }
            let sent = user
                .direct_message(&ctx.http, CreateMessage::new().content(message_text))
                .await
                .context("sending user DM")?;
            log_sent_message(ctx, command, &sent, &target_label_user(user)).await?;
        }
        "channel" => {
            let channel = channel_option(&options, "channel")?;
            let sent = channel
                .id
                .send_message(&ctx.http, CreateMessage::new().content(message_text))
                .await
                .context("sending channel message")?;
            let target = channel
                .name
                .as_ref()
                .map(|name| format!("{name} (<#{}>)", channel.id.get()))
                .unwrap_or_else(|| format!("<#{}>", channel.id.get()));
            log_sent_message(ctx, command, &sent, &target).await?;
        }
        name => return Err(anyhow!("unknown message subcommand {name}")),
    }

    respond_ephemeral(ctx, command, "Sent").await?;
    Ok(())
}

async fn log_sent_message(
    ctx: &Context,
    command: &CommandInteraction,
    sent: &Message,
    target: &str,
) -> Result<()> {
    DAVEBOT
        .send_message(
            &ctx.http,
            CreateMessage::new().embed(sent_message_embed(sent, &command.user, target)),
        )
        .await
        .context("logging sent message")?;
    Ok(())
}

async fn handle_user_verify(ctx: &Context, command: &CommandInteraction) -> Result<()> {
    let Some(ResolvedTarget::User(user, _partial_member)) = command.data.target() else {
        return Err(anyhow!("verify without intro missing target user"));
    };

    let Some(guild_id) = command.guild_id else {
        return Err(anyhow!("verification command outside guild"));
    };
    let member = match guild_id.member(&ctx.http, user.id).await {
        Ok(member) => member,
        Err(_) => {
            respond_ephemeral(ctx, command, "User no longer in the server").await?;
            return Ok(());
        }
    };

    if member.user.id == ctx.cache.current_user().id {
        respond_ephemeral(ctx, command, "You can't verify me!").await?;
        return Ok(());
    }
    if has_role(&member, MEMBER) {
        respond_ephemeral(ctx, command, "User already verified").await?;
        return Ok(());
    }

    respond_with_verify_button(ctx, command, member.user.id).await
}

async fn handle_message_verify(ctx: &Context, command: &CommandInteraction) -> Result<()> {
    let Some(ResolvedTarget::Message(message)) = command.data.target() else {
        return Err(anyhow!("verify command missing target message"));
    };
    verify_member(ctx, command, message.author.id, Some(message)).await
}

async fn respond_with_verify_button(
    ctx: &Context,
    command: &CommandInteraction,
    member_id: UserId,
) -> Result<()> {
    command
        .create_response(
            &ctx.http,
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .content("Are you sure you want to verify this user without an intro?")
                    .ephemeral(true)
                    .components(vec![CreateActionRow::Buttons(vec![
                        CreateButton::new(format!("verify_without_intro:{}", member_id.get()))
                            .label("Verify without intro")
                            .style(ButtonStyle::Primary),
                    ])]),
            ),
        )
        .await
        .context("sending verification confirmation")?;
    Ok(())
}

async fn handle_component(ctx: &Context, component: ComponentInteraction) -> Result<()> {
    let Some(member_id) = component
        .data
        .custom_id
        .strip_prefix("verify_without_intro:")
        .and_then(|id| id.parse::<u64>().ok())
        .map(UserId::new)
    else {
        return Ok(());
    };

    let Some(guild_id) = component.guild_id else {
        return Err(anyhow!("verification component outside guild"));
    };

    let member = match guild_id.member(&ctx.http, member_id).await {
        Ok(member) => member,
        Err(_) => {
            component
                .create_response(
                    &ctx.http,
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new()
                            .content("User no longer in the server")
                            .ephemeral(true),
                    ),
                )
                .await?;
            return Ok(());
        }
    };

    if has_role(&member, MEMBER) {
        component
            .create_response(
                &ctx.http,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                        .content("User already verified")
                        .ephemeral(true),
                ),
            )
            .await?;
        return Ok(());
    }

    let add_role = member.add_role(&ctx.http, MEMBER);
    let remove_role = member.remove_role(&ctx.http, UNVERIFIED);
    let dm = member.user.direct_message(
        &ctx.http,
        CreateMessage::new()
            .content("Congratulations, you're now verified! Welcome to the server!"),
    );
    let modlog = MODLOG.say(
        &ctx.http,
        format!(
            "{} verified {} without an intro",
            component.user.mention(),
            member.mention()
        ),
    );
    let response = component.create_response(
        &ctx.http,
        CreateInteractionResponse::Message(
            CreateInteractionResponseMessage::new()
                .content(thumbsupdirk().mention())
                .ephemeral(true),
        ),
    );

    let (add_result, remove_result, dm_result, modlog_result, response_result) =
        tokio::join!(add_role, remove_role, dm, modlog, response);
    add_result.context("adding member role")?;
    remove_result.context("removing unverified role")?;
    dm_result.context("sending verification DM")?;
    modlog_result.context("logging verification")?;
    response_result.context("responding to verification")?;

    Ok(())
}

async fn verify_member(
    ctx: &Context,
    command: &CommandInteraction,
    user_id: UserId,
    intro_message: Option<&Message>,
) -> Result<()> {
    let Some(guild_id) = command.guild_id else {
        return Err(anyhow!("verification command outside guild"));
    };

    if user_id == ctx.cache.current_user().id {
        respond_ephemeral(ctx, command, "You can't verify me!").await?;
        return Ok(());
    }

    let member = match guild_id.member(&ctx.http, user_id).await {
        Ok(member) => member,
        Err(_) => {
            respond_ephemeral(ctx, command, "User no longer in the server").await?;
            return Ok(());
        }
    };

    if has_role(&member, MEMBER) {
        respond_ephemeral(ctx, command, "User already verified").await?;
        return Ok(());
    }

    if intro_message.is_none() {
        respond_with_verify_button(ctx, command, member.user.id).await?;
        return Ok(());
    }

    let add_role = member.add_role(&ctx.http, MEMBER);
    let remove_role = member.remove_role(&ctx.http, UNVERIFIED);
    let dm = member.user.direct_message(
        &ctx.http,
        CreateMessage::new()
            .content("Congratulations, you're now verified! Welcome to the server!"),
    );
    let verification_response = thumbsupdirk().mention();
    let response = respond_ephemeral(ctx, command, &verification_response);
    let modlog = MODLOG.send_message(
        &ctx.http,
        CreateMessage::new()
            .content(format!(
                "{} verified {}",
                command.user.mention(),
                member.mention()
            ))
            .embed(new_message_embed(
                ctx,
                intro_message.expect("checked above"),
            )),
    );
    let reaction = intro_message
        .expect("checked above")
        .react(&ctx.http, thumbsupdirk().reaction());

    let (add_result, remove_result, dm_result, response_result, modlog_result, reaction_result) =
        tokio::join!(add_role, remove_role, dm, response, modlog, reaction);
    add_result.context("adding member role")?;
    remove_result.context("removing unverified role")?;
    dm_result.context("sending verification DM")?;
    response_result.context("responding to verification")?;
    modlog_result.context("logging verification")?;
    reaction_result.context("reacting to intro")?;

    Ok(())
}

async fn handle_list_unverified(ctx: &Context, command: &CommandInteraction) -> Result<()> {
    let lines = {
        let guild = GUILD
            .to_guild_cached(&ctx.cache)
            .ok_or_else(|| anyhow!("guild is not cached"))?;
        let mut members: Vec<_> = guild
            .members
            .values()
            .filter(|member| !member.user.bot && !has_role(member, MEMBER))
            .filter_map(|member| member.joined_at.map(|joined| (member.user.id, joined)))
            .collect();
        members.sort_by_key(|(_, joined)| *joined);
        members
            .into_iter()
            .map(|(id, joined)| format!("<@{}>: <t:{}:f>", id.get(), joined.unix_timestamp()))
            .collect::<Vec<_>>()
    };

    let chunks = chunk_lines(lines, 2000);
    if chunks.is_empty() {
        respond_ephemeral(ctx, command, "No unverified members.").await?;
        return Ok(());
    }

    let mut chunks = chunks.into_iter();
    if let Some(first) = chunks.next() {
        respond_ephemeral(ctx, command, &first).await?;
    }
    for chunk in chunks {
        followup_ephemeral(ctx, command, &chunk).await?;
    }
    Ok(())
}

async fn handle_member_count(ctx: &Context, command: &CommandInteraction) -> Result<()> {
    let (non_bot, total) = member_counts(ctx);
    respond_ephemeral(
        ctx,
        command,
        &format!("There are currently {non_bot} non-bot members out of {total} in the server."),
    )
    .await
}

async fn handle_poll(ctx: &Context, command: &CommandInteraction) -> Result<()> {
    let Some(ResolvedTarget::Message(message)) = command.data.target() else {
        return Err(anyhow!("poll command missing target message"));
    };

    respond_ephemeral(ctx, command, "Removing reactions...").await?;
    clear_bot_reactions(ctx, message).await?;
    command
        .edit_response(
            &ctx.http,
            EditInteractionResponse::new().content("Reacting..."),
        )
        .await?;

    for emoji in ordered_poll_emojis(&message.content) {
        message.react(&ctx.http, emoji).await?;
    }

    command
        .edit_response(
            &ctx.http,
            EditInteractionResponse::new().content(format!("Done {}", thumbsupdirk().mention())),
        )
        .await?;
    Ok(())
}

async fn handle_unreact(ctx: &Context, command: &CommandInteraction) -> Result<()> {
    let Some(ResolvedTarget::Message(message)) = command.data.target() else {
        return Err(anyhow!("unreact command missing target message"));
    };

    respond_ephemeral(ctx, command, "Removing reactions...").await?;
    clear_bot_reactions(ctx, message).await?;
    command
        .edit_response(
            &ctx.http,
            EditInteractionResponse::new().content(format!("Done {}", thumbsupdirk().mention())),
        )
        .await?;
    Ok(())
}

async fn clear_bot_reactions(ctx: &Context, message: &Message) -> Result<()> {
    let bot_user_id = ctx.cache.current_user().id;
    for reaction in &message.reactions {
        message
            .delete_reaction(&ctx.http, Some(bot_user_id), reaction.reaction_type.clone())
            .await
            .with_context(|| format!("clearing reaction {:?}", reaction.reaction_type))?;
    }
    Ok(())
}

async fn respond_ephemeral(
    ctx: &Context,
    command: &CommandInteraction,
    content: &str,
) -> Result<()> {
    command
        .create_response(
            &ctx.http,
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .content(content)
                    .ephemeral(true),
            ),
        )
        .await
        .context("sending ephemeral response")?;
    Ok(())
}

async fn followup_ephemeral(
    ctx: &Context,
    command: &CommandInteraction,
    content: &str,
) -> Result<()> {
    command
        .create_followup(
            &ctx.http,
            CreateInteractionResponseFollowup::new()
                .content(content)
                .ephemeral(true),
        )
        .await
        .context("sending ephemeral follow-up")?;
    Ok(())
}

fn has_role(member: &Member, role_id: RoleId) -> bool {
    member.roles.contains(&role_id)
}

fn non_bot_member_count(ctx: &Context) -> usize {
    member_counts(ctx).0
}

fn member_counts(ctx: &Context) -> (usize, usize) {
    let Some(guild) = GUILD.to_guild_cached(&ctx.cache) else {
        return (0, 0);
    };
    let total = guild.members.len();
    let non_bot = guild
        .members
        .values()
        .filter(|member| !member.user.bot)
        .count();
    (non_bot, total)
}

fn triggered_emojis(message: &str) -> Vec<CustomEmoji> {
    let casefolded = message.to_lowercase();
    let mut matches = EMOJI_TRIGGERS
        .iter()
        .filter_map(|(emoji, triggers)| {
            triggers
                .iter()
                .filter_map(|trigger| casefolded.find(trigger))
                .min()
                .map(|position| (*emoji, position))
        })
        .collect::<Vec<_>>();
    matches.sort_by_key(|(_, position)| *position);
    matches.into_iter().map(|(emoji, _)| emoji).collect()
}

fn ordered_poll_emojis(message: &str) -> Vec<ReactionType> {
    let mut positions: HashMap<String, (usize, ReactionType)> = HashMap::new();

    for (position, emoji_mention) in custom_emoji_mentions(message) {
        if let Ok(reaction) = ReactionType::try_from(emoji_mention.as_str()) {
            positions
                .entry(emoji_mention)
                .or_insert((position, reaction));
        }
    }

    for (position, emoji) in unicode_emojis(message) {
        positions
            .entry(emoji.clone())
            .or_insert((position, ReactionType::Unicode(emoji)));
    }

    let mut ordered = positions.into_values().collect::<Vec<_>>();
    ordered.sort_by_key(|(position, _)| *position);
    ordered.into_iter().map(|(_, emoji)| emoji).collect()
}

fn custom_emoji_mentions(message: &str) -> Vec<(usize, String)> {
    static CUSTOM_EMOJI_RE: OnceLock<Regex> = OnceLock::new();
    let custom_emoji_re =
        CUSTOM_EMOJI_RE.get_or_init(|| Regex::new(r"<a?:[A-Za-z0-9_]+:[0-9]+>").unwrap());

    custom_emoji_re
        .find_iter(message)
        .map(|found| (found.start(), found.as_str().to_owned()))
        .collect()
}

fn unicode_emojis(message: &str) -> Vec<(usize, String)> {
    static EMOJI_RE: OnceLock<Regex> = OnceLock::new();
    let emoji_re = EMOJI_RE.get_or_init(|| {
        Regex::new(
            r"[\p{Emoji_Presentation}\p{Emoji}\u{FE0F}](?:\u{200D}[\p{Emoji_Presentation}\p{Emoji}\u{FE0F}])*",
        )
        .expect("valid emoji regex")
    });

    emoji_re
        .find_iter(message)
        .map(|found| (found.start(), found.as_str().to_owned()))
        .collect()
}

fn new_message_embed(ctx: &Context, message: &Message) -> CreateEmbed {
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

fn sent_message_embed(message: &Message, author: &User, target: &str) -> CreateEmbed {
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

fn target_label_user(user: &User) -> String {
    format!("{} ({})", user.tag(), user.mention())
}

fn string_option<'a>(
    options: &'a [serenity::all::ResolvedOption<'a>],
    name: &str,
) -> Result<&'a str> {
    options
        .iter()
        .find(|option| option.name == name)
        .and_then(|option| match option.value {
            ResolvedValue::String(value) => Some(value),
            _ => None,
        })
        .ok_or_else(|| anyhow!("missing string option {name}"))
}

fn user_option<'a>(
    options: &'a [serenity::all::ResolvedOption<'a>],
    name: &str,
) -> Result<&'a User> {
    options
        .iter()
        .find(|option| option.name == name)
        .and_then(|option| match option.value {
            ResolvedValue::User(user, _) => Some(user),
            _ => None,
        })
        .ok_or_else(|| anyhow!("missing user option {name}"))
}

fn channel_option<'a>(
    options: &'a [serenity::all::ResolvedOption<'a>],
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

fn chunk_lines(lines: Vec<String>, limit: usize) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut pending = Vec::new();
    let mut pending_len = 0;

    for line in lines {
        let separator_len = usize::from(!pending.is_empty());
        if !pending.is_empty() && pending_len + separator_len + line.len() > limit {
            chunks.push(pending.join("\n"));
            pending.clear();
            pending_len = 0;
        }

        pending_len += line.len() + usize::from(!pending.is_empty());
        pending.push(line);
    }

    if !pending.is_empty() {
        chunks.push(pending.join("\n"));
    }

    chunks
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "calibornia_discord_bot=info,serenity=info".into()),
        )
        .init();

    let config = Config::from_env()?;
    let mut client = Client::builder(config.discord_token, GatewayIntents::all())
        .event_handler(Handler)
        .await
        .context("creating Discord client")?;

    client.start().await.context("running Discord client")
}
