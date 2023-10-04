use chatgpt::prelude::*;
use serenity::{async_trait, futures::StreamExt, model::prelude::Message, prelude::Context};
use tracing::{error, info};

use crate::bot::Bot;

const DIRECTED_PROMPT: &str = "
You are the Animeboys Bot. Your main purpose it to help members of the Animeboys Discord server debug their code.
All code blocks should be within ```. If there is no ``` within the request, then you should say that the request is invalid.
Please start every response with ``` so that the Discord server can format the response properly.
Please start every response with 'Thanks for using Animeboys Bot!'
Please finish every response with 'Feel free to ask me another question using the $ai command!'
";

#[async_trait]
pub trait ChatGPTCommands {
    async fn print_ai_help(&self) -> String;
    async fn animeboys_ai_handler(&self, command: Vec<&str>, ctx: &Context, msg: &Message);
}

pub struct AnimeboysAI {
    client: ChatGPT,
    conversation: Conversation,
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

    pub fn clone(&self) -> Self {
        let client = self.client.clone();
        let conversation = client.new_conversation_directed(DIRECTED_PROMPT);
        Self {
            client,
            conversation,
        }
    }
}

#[async_trait]
impl ChatGPTCommands for Bot {
    async fn animeboys_ai_handler(&self, command: Vec<&str>, ctx: &Context, msg: &Message) {
        let typing = msg.channel_id.start_typing(&ctx.http).unwrap();
        let action = match command.get(0) {
            Some(action) => action.trim(),
            None => {
                if let Err(e) = msg
                    .channel_id
                    .say(&ctx.http, "Please provide an action.")
                    .await
                {
                    error!("Error sending message: {:?}", e);
                }
                return;
            }
        };
        info!("Action: {}", action);
        match action {
            // Check debug and debug\n
            "debug" => {
                let code = {
                    // Combine the rest of the command into a single string
                    let mut code = String::new();
                    for word in command.iter().skip(1) {
                        code.push_str(word);
                        code.push(' ');
                    }
                    code
                };
                let ai = &mut self.ai.clone();
                let res = ai.debug(&code).await;
                // if the response is too long, then send it in multiple messages
                if res.bytes().len() > 2000 {
                    let res = res.bytes().collect::<Vec<u8>>();
                    for chunk in res.chunks(2000) {
                        let res = String::from_utf8(chunk.to_vec()).unwrap();
                        if let Err(e) = msg.channel_id.say(&ctx.http, res).await {
                            error!("Error sending message: {:?}", e);
                        }
                    }
                } else {
                    if let Err(e) = msg.channel_id.say(&ctx.http, res).await {
                        error!("Error sending message: {:?}", e);
                    }
                }
            }
            "help" => {
                if let Err(e) = msg
                    .channel_id
                    .say(&ctx.http, self.print_ai_help().await)
                    .await
                {
                    error!("Error sending message: {:?}", e);
                }
            }
            _ => {
                if let Err(e) = msg
                    .channel_id
                    .say(
                        &ctx.http,
                        "Invalid command. Please use $ai help for a list of commands.",
                    )
                    .await
                {
                    error!("Error sending message: {:?}", e);
                }
            }
        }
        typing.stop().unwrap();
    }

    async fn print_ai_help(&self) -> String {
        return "
        Commands for Animeboys AI:
        $ai debug <code block> - Debugs the code block. Code block must be within ```.
        $ai help - Prints this help message.
        "
        .to_string();
    }
}
