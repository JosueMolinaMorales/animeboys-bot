use std::collections::HashSet;

use crate::{
    aws::{command::MINECRAFTCOMMANDS_GROUP, ec2::Ec2Client},
    chatgpt::animeboys_ai::{AnimeboysAI, AICOMMANDS_GROUP},
    wz::WZCOMMANDS_GROUP,
};
use serenity::{
    async_trait,
    framework::{
        standard::{
            help_commands,
            macros::{group, help},
            CommandGroup, DispatchError, HelpOptions,
        },
        StandardFramework,
    },
    model::prelude::{GuildChannel, PartialGuildChannel, UserId},
};
use serenity::{framework::standard::macros::command, model::gateway::Ready};
use serenity::{framework::standard::macros::hook, model::channel::Message};
use serenity::{framework::standard::Args, model::prelude::Member};
use serenity::{framework::standard::CommandResult, prelude::*};
use tracing::{error, info};

const MEMBER_ROLE_ID: u64 = 342563599572664321;
struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn thread_update(&self, ctx: Context, thread: GuildChannel) {
        info!("Updating thread: {}", thread.id);
        // If the thread is locked, then remove it from the AI
        if thread.thread_metadata.unwrap().locked {
            // Delete the thread from the AI
            let mut data = ctx.data.write().await;
            let ai = data.get_mut::<AnimeboysAI>().unwrap();
            ai.remove_thread(&thread.id).await;
        }
    }

    async fn thread_delete(&self, ctx: Context, thread: PartialGuildChannel) {
        info!("Deleting thread: {}", thread.id);
        // Delete the thread from the AI
        let mut data = ctx.data.write().await;
        let ai = data.get_mut::<AnimeboysAI>().unwrap();
        ai.remove_thread(&thread.id).await;
    }

    async fn guild_member_addition(&self, ctx: Context, new_member: Member) {
        // Send the user a welcome message
        if let Err(e) = new_member
            .user
            .direct_message(&ctx.http, |m| {
                m.content(format!(
                    "Welcome to the Animeboys server, {}! Please read the rules and enjoy your stay!",
                    new_member.user.name
                ))
            })
            .await
        {
            error!("Error sending message: {:?}", e);
        }
        // Get the guild_id
        let guild_id = new_member.guild_id.0;
        // Get the user_id
        let user_id = new_member.user.id.0;
        // Assign them to the member role
        if let Err(err) = ctx
            .http
            .add_member_role(
                guild_id,
                user_id,
                MEMBER_ROLE_ID,
                Some("Animeboys Bot Added Role to User"),
            )
            .await
        {
            error!("Error assigning role to user: {} err: {:?}", user_id, err);
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[group]
#[commands(ping)]
struct General;

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id.say(&ctx.http, "Pong!").await?;

    Ok(())
}

// The framework provides two built-in help commands for you to use.
// But you can also make your own customized help command that forwards
// to the behaviour of either of them.
#[help]
// This replaces the information that a user can pass
// a command-name as argument to gain specific information about it.
#[individual_command_tip = "Hello! こんにちは！Hola! Bonjour! 您好! 안녕하세요~\n\n\
If you want more information about a specific command, just pass the command as argument."]
// Some arguments require a `{}` in order to replace it with contextual information.
// In this case our `{}` refers to a command's name.
#[command_not_found_text = "Could not find: `{}`."]
// On another note, you can set up the help-menu-filter-behaviour.
// Here are all possible settings shown on all possible options.
// First case is if a user lacks permissions for a command, we can hide the command.
#[lacking_permissions = "Hide"]
// If the user is nothing but lacking a certain role, we just display it hence our variant is `Nothing`.
#[lacking_role = "Nothing"]
// The last `enum`-variant is `Strike`, which ~~strikes~~ a command.
#[wrong_channel = "Strike"]
// Serenity will automatically analyse and generate a hint/tip explaining the possible
// cases of ~~strikethrough-commands~~, but only if
// `strikethrough_commands_tip_in_{dm, guild}` aren't specified.
// If you pass in a value, it will be displayed instead.
async fn my_help(
    context: &Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    let _ = help_commands::with_embeds(context, msg, args, help_options, groups, owners).await?;
    Ok(())
}

#[hook]
async fn unknown_command(ctx: &Context, msg: &Message, unknown_command_name: &str) {
    msg.channel_id
        .say(
            &ctx.http,
            format!("Could not find command named '{}'", unknown_command_name),
        )
        .await
        .unwrap();
}

#[hook]
async fn dispatch_error(ctx: &Context, msg: &Message, error: DispatchError, _command_name: &str) {
    match error {
        DispatchError::NotEnoughArguments { min, given } => {
            msg.channel_id
                .say(
                    &ctx.http,
                    &format!(
                        "Command expected at least {} arguments, found: {}",
                        min, given
                    ),
                )
                .await
                .unwrap();
        }
        DispatchError::TooManyArguments { max, given } => {
            msg.channel_id
                .say(
                    &ctx.http,
                    &format!(
                        "Command expected at most {} arguments, found: {}",
                        max, given
                    ),
                )
                .await
                .unwrap();
        }
        _ => {
            msg.channel_id
                .say(&ctx.http, "Something went wrong")
                .await
                .unwrap();
            // Contact the dev that something went wrong
            let dm = 1155886697582690345;
            ctx.http
                .send_message(
                    dm,
                    &serde_json::json!({
                        "content": format!("Error: {:#?}", error),
                    }),
                )
                .await
                .unwrap();
        }
    }
}

#[hook]
async fn before(_ctx: &Context, msg: &Message, command_name: &str) -> bool {
    info!(
        "Got command '{}' by user '{}'",
        command_name, msg.author.name
    );

    // true -> continue, false -> do not continue
    true
}

#[hook]
async fn normal_message(ctx: &Context, msg: &Message) {
    info!("Got message '{}'", msg.content);
    // Check to see if the message was sent in a thread
    let mut data = ctx.data.write().await;
    let ai = data.get_mut::<AnimeboysAI>().unwrap();

    if ai.does_thread_exist(&msg.channel_id) {
        info!("Message was sent in a thread");
        // Start Typing
        let typing = ctx.http.start_typing(msg.channel_id.0).unwrap();
        // Send the message to the AI
        let response = ai.send_message(&msg.content, &msg.channel_id).await;

        // Get the guild channel from the channel id
        let channel = ctx
            .http
            .get_channel(msg.channel_id.0)
            .await
            .unwrap()
            .guild()
            .unwrap();

        // Send the response to the thread
        send_message_in_streams(ctx, channel, response)
            .await
            .unwrap();
        // Stop typing
        drop(typing);
    }
}

/// Sends a message in multiple streams
/// If the message is too long, then it will be split into multiple messages
/// and sent in multiple streams
pub async fn send_message_in_streams(
    ctx: &Context,
    channel: GuildChannel,
    msg: String,
) -> CommandResult {
    if msg.bytes().len() > 2000 {
        let msg = msg.bytes().collect::<Vec<u8>>();
        for chunk in msg.chunks(2000) {
            let msg = String::from_utf8(chunk.to_vec()).unwrap();
            channel.send_message(&ctx.http, |m| m.content(msg)).await?;
        }
    } else {
        channel.send_message(&ctx.http, |m| m.content(msg)).await?;
    }
    Ok(())
}

/// Create Bot Framework
pub fn create_framework() -> StandardFramework {
    let framework = StandardFramework::new()
        .configure(|c| c.prefix("$"))
        // Set a function that's called whenever an attempted command-call's
        // command could not be found.
        .unrecognised_command(unknown_command)
        // Set a function that's called whenever a command's execution didn't complete for one
        // reason or another. For example, when a user has exceeded a rate-limit or a command
        // can only be performed by the bot owner.
        .on_dispatch_error(dispatch_error)
        .normal_message(normal_message)
        .before(before)
        .help(&MY_HELP)
        .group(&GENERAL_GROUP)
        .group(&AICOMMANDS_GROUP)
        .group(&MINECRAFTCOMMANDS_GROUP)
        .group(&WZCOMMANDS_GROUP);

    framework
}

pub async fn create_bot(
    token: String,
    intents: GatewayIntents,
    api_key: String,
    instance_id: String,
) -> Client {
    let framework = create_framework();
    let client = Client::builder(&token, intents)
        .event_handler(Handler)
        .framework(framework)
        .type_map_insert::<AnimeboysAI>(AnimeboysAI::new(&api_key))
        .type_map_insert::<Ec2Client>(Ec2Client::new(instance_id).await)
        .await
        .expect("Err creating client");

    client
}
