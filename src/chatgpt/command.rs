use serenity::{
    framework::standard::{
        macros::{check, command, group},
        Args, CommandError, CommandOptions, CommandResult, Reason,
    },
    model::prelude::{Channel, ChannelType, Message},
    prelude::Context,
};

use crate::{bot, chatgpt::animeboys_ai::AnimeboysAI};

#[group("AI Commands")]
#[prefixes("ai")]
#[description("Commands for using the AI")]
#[summary("Commands for using the AI")]
#[commands(debug, chat, help, stop)]
#[default_command(chat)]
struct AICommands;

#[command]
#[min_args(0)]
#[max_args(0)]
/// Stop stops the current conversation
/// If the conversation is in a thread, then the thread will be deleted
/// If the conversation is in a DM, then the conversation will be deleted
async fn stop(ctx: &Context, msg: &Message) -> CommandResult {
    let mut data = ctx.data.write().await;
    let ai = data.get_mut::<AnimeboysAI>().unwrap();

    // Check to see if the message was sent in an existing conversation
    let channel = msg.channel_id.to_channel(&ctx.http).await?;
    if !ai.does_conversation_exist(&channel.id()) {
        msg.channel_id
            .say(&ctx.http, "No conversation exists")
            .await?;
        return Ok(());
    }

    // Remove the conversation from the ai struct
    ai.remove_conversation(&channel.id()).await;

    // Check to see if the conversation is in a thread
    // If it is, then delete the thread
    if let Channel::Guild(thread) = channel {
        msg.channel_id.say(&ctx.http, "Deleting thread...").await?;
        thread.delete(&ctx.http).await?;
        return Ok(());
    }

    // Check to see if the conversation is in a DM
    // If it is, then send message to user
    if let Channel::Private(channel) = channel {
        channel.say(&ctx.http, "Thanks for using the Animeboys AI! This conversation will now be deleted. To restart the conversation, run `$ai chat`").await?;
        return Ok(());
    }

    Ok(())
}

#[command]
#[description("Displays the help message")]
#[aliases("h")]
#[min_args(0)]
#[max_args(0)]
async fn help(ctx: &Context, msg: &Message) -> CommandResult {
    let help = "
    >>> **AI Commands**
    `$ai debug <code block>` - Debugs the given code
    `$ai chat` - Starts a new conversation with the AI
    `$ai stop` - Stops the current conversation
    `$ai help` - Displays this help message
    ";
    msg.channel_id.say(&ctx.http, help).await?;
    Ok(())
}

#[command]
#[description("Chat with the AI")]
#[usage("chat")]
/// Chat creates a new thread with the AI where you can chat with it
async fn chat(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let mut data = ctx.data.write().await;
    let ai = data.get_mut::<AnimeboysAI>().unwrap();

    let channel = check_for_conversation(ai, ctx, msg).await?;

    // Save thread
    let res = ai
        .create_conversation(&msg.author.name, &channel.id())
        .await;

    // Start Typing
    let typing = ctx.http.start_typing(channel.id().0)?;

    // Send intro message, that the ai is ready
    bot::send_message_in_streams(ctx, channel, res).await?;

    // Stop Typing
    drop(typing);
    Ok(())
}

/// Checks to see if a conversation exists within the ai struct
/// If it does not exist, then it creates a new thread
/// If it does exist, then it returns the channel
async fn check_for_conversation(
    ai: &AnimeboysAI,
    ctx: &Context,
    msg: &Message,
) -> Result<Channel, CommandError> {
    // Check to see if the message was sent in a thread
    let channel;
    if !ai.does_conversation_exist(&msg.channel_id) {
        // Check to see if the message was sent in a DM
        // if so, then use the DM channel
        if msg.is_private() {
            channel = msg.channel(&ctx.http).await?;
        } else {
            // Create a new thread
            let thread = msg
                .channel_id
                .create_public_thread(&ctx.http, msg.id, |t| {
                    t.name(format!("Chat Thread for {}", msg.author.name))
                        .auto_archive_duration(60)
                        .kind(ChannelType::PublicThread)
                })
                .await?;
            channel = Channel::Guild(thread);
        }
    } else {
        // Get conversation from id
        channel = ctx.http.get_channel(msg.channel_id.0).await?;
    }
    Ok(channel)
}

#[command]
#[usage("debug <code block>")]
#[example("debug ```print('Hello World!')```")]
#[checks(CodeBlock)]
/// Debug creates a thread (if within a server) and debugs the given code
/// The code must be in a code block
/// After the thread is created (if within a server) you can continue to converse with
/// the AI in the thread or DM without having to use the $ai command
async fn debug(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let mut data = ctx.data.write().await;
    let ai = data.get_mut::<AnimeboysAI>().unwrap();

    // Check to see if the message was sent in an existing conversation
    let channel = check_for_conversation(ai, ctx, msg).await?;
    // Start Typing
    let typing = ctx.http.start_typing(channel.id().0)?;

    let code = args.rest();

    let res = ai.debug(&code, &channel.id()).await;

    // if the response is too long, then send it in multiple messages
    bot::send_message_in_streams(ctx, channel, res).await?;

    // Stop typing
    drop(typing);
    Ok(())
}

#[check]
#[name = "CodeBlock"]
async fn has_code_block(
    _: &Context,
    msg: &Message,
    _: &mut Args,
    _: &CommandOptions,
) -> Result<(), Reason> {
    // Message must contain ``` twice to be a valid code block
    if msg.content.matches("```").count() < 2 {
        return Err(Reason::User("Invalid code block".into()));
    }
    Ok(())
}
