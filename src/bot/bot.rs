use crate::aws::{self, ecs_commands::EcsCommands};
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::prelude::Member;
use serenity::prelude::*;
use tracing::{error, info};

const CHANNEL_ID: u64 = 1081598245358276721;
pub struct Bot {
    /// The instance id of the ec2 instance
    pub instance_id: String,
    /// The profile to use when connecting to aws
    pub ec2_client: aws_sdk_ec2::Client,
}

impl Bot {
    pub async fn new(instance_id: String) -> Bot {
        let client = aws::create_ecs_client().await;
        Bot {
            instance_id,
            ec2_client: client,
        }
    }

    pub async fn print_help(&self) -> String {
        "
    Welcome to the Animeboys Bot! Here are the commands you can use:
        $mc-help - Displays the help message for managing the minecraft server
        $help    - Displays this message
        "
        .into()
    }
}

#[async_trait]
impl EventHandler for Bot {
    async fn guild_member_addition(&self, ctx: Context, new_member: Member) {}

    async fn message(&self, ctx: Context, msg: Message) {
        // Ignore messages from self
        if msg.author.bot {
            return;
        }
        if msg.channel_id.0 != CHANNEL_ID {
            info!("Message sent in wrong channel");
            return;
        }
        info!("Message received from {}", msg.author.tag());

        info!("Message received: {}", msg.content);

        // Get the prefix of the message
        let command = msg.content.split('-').collect::<Vec<&str>>();
        let prefix = command[0];
        let command = command[1];
        match prefix.to_ascii_lowercase().as_str() {
            "$mc" => {
                self.ec2_handler(command, &ctx, &msg).await;
            }
            "$help" => {
                if let Err(e) = msg.channel_id.say(&ctx.http, self.print_help().await).await {
                    error!("Error sending message: {:?}", e);
                }
            }
            "$hi" => {
                if let Err(e) = msg
                    .channel_id
                    .say(&ctx.http, format!("Hello {}!", msg.author.name))
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
                        "Unknown command. Try $help for a list of commands.",
                    )
                    .await
                {
                    error!("Error sending message: {:?}", e);
                }
            }
        }
    }

    async fn ready(&self, _ctx: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
    }
}
