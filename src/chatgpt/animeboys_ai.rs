use std::collections::HashMap;

use chatgpt::{
    prelude::{ChatGPT, Conversation, ModelConfigurationBuilder},
    types::{ChatMessage, ResponseChunk, Role},
};
use serenity::{futures::StreamExt, model::prelude::ChannelId, prelude::TypeMapKey};
use tracing::{error, info};

const DEBUG_DIRECTED_PROMPT: &str = "
You are the Animeboys Bot. Your main purpose it to help members of the Animeboys Discord server debug their code.
The conversation will start with a user requesting help with their code. You will then respond with a message that will help the user debug their code.
After this, you will be placed in a thread with the user where you can continue to help them with their code.
When you respond, always end your message with 'Thank you for using the Animeboys Bot! Is there anything else I can assist with?'.
If the user responds with 'Yes', then you will continue to help them with their code. If the user responds with 'No', then you will end the conversation.
";

const QUESTION_DIRECTED_PROMPT: &str = "
You are the Animeboys Bot. Your main purpose it to help members of the Animeboys Discord server. 
In this conversation you will help members by answering their questions. After every couple of messages, please
remind the user that they can end the conversation by sending `$ai stop`.
";

pub struct AnimeboysAI {
    client: ChatGPT,
    /// A map of channel ids to conversations
    /// The key is the channel id of the thread or a private message
    conversations: HashMap<ChannelId, Conversation>,
}

impl TypeMapKey for AnimeboysAI {
    type Value = AnimeboysAI;
}

impl AnimeboysAI {
    pub fn new(api_key: &str) -> Self {
        let client = ChatGPT::new_with_config(
            api_key,
            ModelConfigurationBuilder::default()
                .timeout(std::time::Duration::from_secs(120))
                .build()
                .unwrap(),
        )
        .expect("Failed to create ChatGPT client");

        Self {
            client,
            conversations: HashMap::new(),
        }
    }

    pub async fn debug(&mut self, code: &str, channel_id: &ChannelId) -> String {
        // Create a new conversation if one does not exist
        let conversation = self
            .conversations
            .entry(*channel_id)
            .or_insert_with(|| self.client.new_conversation_directed(DEBUG_DIRECTED_PROMPT));

        let res = AnimeboysAI::get_message_from_stream(Role::User, conversation, code).await;
        res
    }

    pub async fn send_message(&mut self, message: &str, channel_id: &ChannelId) -> String {
        // Create a new conversation if one does not exist
        let mut conversation = self.conversations.entry(*channel_id).or_insert_with(|| {
            self.client
                .new_conversation_directed(QUESTION_DIRECTED_PROMPT)
        });
        info!("Conversation history: {:#?}", conversation.history);

        let res =
            AnimeboysAI::get_message_from_stream(Role::User, &mut conversation, message).await;
        res
    }

    pub async fn create_conversation(&mut self, user: &str, channel_id: &ChannelId) -> String {
        // Create a new conversation if one does not exist
        let conversation = self.conversations.entry(*channel_id).or_insert_with(|| {
            self.client
                .new_conversation_directed(QUESTION_DIRECTED_PROMPT)
        });
        info!("Conversation history: {:#?}", conversation.history);

        let res = AnimeboysAI::get_message_from_stream(
            Role::User,
            conversation,
            &format!(
                "Hello bot! I am {}! I Started this thread to chat with you!",
                user
            ),
        )
        .await;

        res
    }

    pub fn does_conversation_exist(&self, channel_id: &ChannelId) -> bool {
        self.conversations.contains_key(channel_id)
    }

    pub async fn remove_conversation(&mut self, channel_id: &ChannelId) {
        self.conversations.remove(channel_id);
    }

    /// Sends a message to the AI and returns the response
    /// # Arguments
    /// * `conversation` - The conversation to send the message to
    /// * `message` - The message to send to the AI
    /// * `role` - The role of the message
    /// # Returns
    /// The response from the AI
    async fn get_message_from_stream(
        role: Role,
        conversation: &mut Conversation,
        message: &str,
    ) -> String {
        let mut stream = match conversation
            .send_role_message_streaming(role, message)
            .await
        {
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

        // Save the response to the conversation history
        conversation.history.push(ChatMessage {
            role: Role::Assistant,
            content: res.clone(),
            function_call: None,
        });

        res
    }
}
