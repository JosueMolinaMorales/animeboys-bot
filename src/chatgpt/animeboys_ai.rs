use std::collections::HashMap;

use chatgpt::{
    prelude::{ChatGPT, Conversation, ModelConfigurationBuilder},
    types::{ChatMessage, ResponseChunk},
};
use serenity::{
    framework::standard::{
        macros::{check, command, group},
        Args, CommandOptions, CommandResult, Reason,
    },
    futures::StreamExt,
    model::prelude::{ChannelId, ChannelType, Message},
    prelude::{Context, TypeMapKey},
};
use tracing::{error, info};

use crate::bot;

#[group("AI Commands")]
#[prefixes("ai")]
#[description("Commands for using the AI")]
#[summary("Commands for using the AI")]
#[commands(debug, help, reset)]
struct AICommands;

#[command]
#[description("Resets the AI discussion")]
#[help_available(false)]
#[min_args(0)]
#[max_args(0)]
#[required_permissions("ADMINISTRATOR")]
async fn reset(ctx: &Context, msg: &Message) -> CommandResult {
    let typing = msg.channel_id.start_typing(&ctx.http)?;
    // TODO: Reset the conversation
    msg.channel_id
        .say(&ctx.http, "AI Discussion has been reset")
        .await?;
    drop(typing);
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
    ";
    msg.channel_id.say(&ctx.http, help).await?;
    Ok(())
}

#[command]
#[description("Debugs the given code")]
#[usage("debug <code block>")]
#[example("debug ```print('Hello World!')```")]
#[checks(CodeBlock)]
async fn debug(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let mut data = ctx.data.write().await;
    let ai = data.get_mut::<AnimeboysAI>().unwrap();

    // Check to see if the message was sent in a thread
    let thread;
    if !ai.does_thread_exist(&msg.channel_id) {
        // Create a new thread
        thread = msg
            .channel_id
            .create_public_thread(&ctx.http, msg.id, |t| {
                t.name(format!("Debug Thread for {}", msg.author.name))
                    .auto_archive_duration(60)
                    .kind(ChannelType::PublicThread)
            })
            .await?;
    } else {
        // Get thread from id
        thread = ctx
            .http
            .get_channel(msg.channel_id.0)
            .await?
            .guild()
            .unwrap(); // TODO: Remove unwrap
    }

    // Start Typing
    let typing = ctx.http.start_typing(thread.id.0)?;

    let code = args.rest();

    let res = ai.debug(&code, &thread.id).await;

    // if the response is too long, then send it in multiple messages
    bot::send_message_in_streams(ctx, thread, res).await?;

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

const DEBUG_DIRECTED_PROMPT: &str = "
You are the Animeboys Bot. Your main purpose it to help members of the Animeboys Discord server debug their code.
The conversation will start with a user requesting help with their code. You will then respond with a message that will help the user debug their code.
After this, you will be placed in a thread with the user where you can continue to help them with their code.
When you respond, always end your message with 'Thank you for using the Animeboys Bot! Is there anything else I can assist with?'.
If the user responds with 'Yes', then you will continue to help them with their code. If the user responds with 'No', then you will end the conversation.
";

const QUESTION_DIRECTED_PROMPT: &str = "
You are the Animeboys Bot. Your main purpose it to help members of the Animeboys Discord server. In this conversation you will help members by answering their questions.

";

pub struct AnimeboysAI {
    client: ChatGPT,
    threads: HashMap<ChannelId, Conversation>,
}

impl TypeMapKey for AnimeboysAI {
    type Value = AnimeboysAI;
}

impl AnimeboysAI {
    pub fn new(api_key: &str) -> Self {
        let client = ChatGPT::new_with_config(
            api_key,
            ModelConfigurationBuilder::default()
                .timeout(std::time::Duration::from_secs(60))
                .build()
                .unwrap(),
        )
        .expect("Failed to create ChatGPT client");

        Self {
            client,
            threads: HashMap::new(),
        }
    }

    pub async fn debug(&mut self, code: &str, channel_id: &ChannelId) -> String {
        // Create a new conversation if one does not exist
        let conversation = self
            .threads
            .entry(*channel_id)
            .or_insert_with(|| self.client.new_conversation_directed(DEBUG_DIRECTED_PROMPT));

        let res = AnimeboysAI::get_message_from_stream(conversation, code).await;
        res
    }

    pub async fn send_message(&mut self, message: &str, channel_id: &ChannelId) -> String {
        // Create a new conversation if one does not exist
        let mut conversation = self.threads.entry(*channel_id).or_insert_with(|| {
            self.client
                .new_conversation_directed(QUESTION_DIRECTED_PROMPT)
        });
        info!("Conversation history: {:#?}", conversation.history);

        let res = AnimeboysAI::get_message_from_stream(&mut conversation, message).await;
        res
    }

    pub fn does_thread_exist(&self, channel_id: &ChannelId) -> bool {
        self.threads.contains_key(channel_id)
    }

    pub async fn remove_thread(&mut self, channel_id: &ChannelId) {
        self.threads.remove(channel_id);
    }

    async fn get_message_from_stream(conversation: &mut Conversation, message: &str) -> String {
        let mut stream = match conversation.send_message_streaming(message).await {
            Ok(stream) => stream,
            Err(e) => {
                error!("Error sending message: {:?}", e);
                return "There was an error processing your request. Please try again later!"
                    .to_string();
            }
        };
        // Build output from stream
        let mut output: Vec<ResponseChunk> = Vec::new();
        while let Some(chunck) = stream.next().await {
            match chunck {
                ResponseChunk::Content {
                    delta,
                    response_index,
                } => output.push(ResponseChunk::Content {
                    delta,
                    response_index,
                }),
                other => output.push(other),
            }
        }
        let messages = ChatMessage::from_response_chunks(output);
        let mut res = String::new();
        for message in messages {
            res.push_str(&message.content);
        }

        res
    }
}
