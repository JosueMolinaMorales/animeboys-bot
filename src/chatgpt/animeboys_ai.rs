use chatgpt::{
    prelude::{ChatGPT, Conversation, ModelConfigurationBuilder},
    types::{ChatMessage, ResponseChunk},
};
use serenity::{
    framework::standard::{
        macros::{command, group},
        Args, CommandResult,
    },
    futures::StreamExt,
    model::prelude::Message,
    prelude::{Context, TypeMapKey},
};
use tracing::error;

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
    let mut data = ctx.data.write().await;
    let ai = data.get_mut::<AnimeboysAI>().unwrap();
    ai.reset().await;
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
async fn debug(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    // Start typing
    let typing = msg.channel_id.start_typing(&ctx.http)?;

    let code = args.rest();
    let mut data = ctx.data.write().await;

    let ai = data.get_mut::<AnimeboysAI>().unwrap();
    let res = ai.debug(&code).await;
    // if the response is too long, then send it in multiple messages
    if res.bytes().len() > 2000 {
        let res = res.bytes().collect::<Vec<u8>>();
        for chunk in res.chunks(2000) {
            let res = String::from_utf8(chunk.to_vec()).unwrap();
            msg.channel_id.say(&ctx.http, res).await?;
        }
    } else {
        msg.channel_id.say(&ctx.http, res).await?;
    }

    // Stop typing
    drop(typing);
    Ok(())
}

const DIRECTED_PROMPT: &str = "
You are the Animeboys Bot. Your main purpose it to help members of the Animeboys Discord server debug their code.
All code blocks should be within ```. If there is no ``` within the request, then you should say that the request is invalid.
Please start every response with ``` so that the Discord server can format the response properly.
Please start every response with 'Thanks for using Animeboys Bot!'
Please finish every response with 'Feel free to ask me another question using the $ai command!'
";

pub struct AnimeboysAI {
    client: ChatGPT,
    conversation: Conversation,
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

        let conversation = client.new_conversation_directed(DIRECTED_PROMPT);
        Self {
            client,
            conversation,
        }
    }

    pub async fn debug(&mut self, code: &str) -> String {
        let mut stream = match self.conversation.send_message_streaming(code).await {
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

    pub async fn reset(&mut self) {
        self.conversation = self.client.new_conversation_directed(DIRECTED_PROMPT);
    }
}
