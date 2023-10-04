use crate::{
    aws::{self, ecs_commands::EcsCommands},
    chatgpt::{AnimeboysAI, ChatGPTCommands},
    wz::WzLoadoutCommands,
};
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
    /// The role assigned to all new members of the server
    pub member_role: u64,
    /// The chatgpt ai
    pub ai: AnimeboysAI,
}

impl Bot {
    pub async fn new(instance_id: String, api_key: String) -> Bot {
        let client = aws::create_ecs_client().await;
        let ai = AnimeboysAI::new(&api_key);
        Bot {
            instance_id,
            ec2_client: client,
            member_role: 342563599572664321,
            ai,
        }
    }

    pub async fn print_help(&self) -> String {
        "
    Welcome to the Animeboys Bot! Here are the commands you can use:
        `$mc help` - Displays the help message for managing the minecraft server
        `$wz help` - Displays the help message for managing the warzone server
        `$ai help` - Displays the help message for using the AI
        `$help`    - Displays this message
        "
        .into()
    }
}

#[async_trait]
impl EventHandler for Bot {
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
                self.member_role,
                Some("Animeboys Bot Added Role to User"),
            )
            .await
        {
            error!("Error assigning role to user: {} err: {:?}", user_id, err);
        }
    }

    async fn message(&self, ctx: Context, msg: Message) {
        // Ignore messages from self
        if msg.author.bot {
            return;
        }
        if msg.channel_id.0 != CHANNEL_ID && !msg.is_private() {
            info!("Message sent in wrong channel");
            return;
        }
        info!("Message received from {}", msg.author.tag());

        info!("Message received: {}", msg.content);

        // Get the prefix of the message
        let mut command = msg.content.split(' ').collect::<Vec<&str>>();
        let prefix = command[0];
        match prefix.to_ascii_lowercase().as_str() {
            "$mc" => {
                // Ensure there is a command after the prefix
                if command.len() < 2 {
                    if let Err(e) = msg
                        .channel_id
                        .say(
                            &ctx.http,
                            "Unknown command. Try $mc help for a list of commands.",
                        )
                        .await
                    {
                        error!("Error sending message: {:?}", e);
                    }
                    return;
                }
                let command = command[1];
                self.ec2_handler(command, &ctx, &msg).await;
            }
            "$wz" => {
                command.remove(0);
                self.wz_loadout_handler(command, &ctx, &msg).await;
            }
            "$ai" => {
                command.remove(0);
                self.animeboys_ai_handler(command, &ctx, &msg).await;
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
            prefix => {
                if !prefix.starts_with('$') {
                    return;
                }
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
