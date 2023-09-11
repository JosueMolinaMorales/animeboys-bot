use std::time::Duration;

use crate::bot::Bot;
use serenity::{async_trait, model::channel::Message, prelude::Context};
use tracing::error;

use super::Ec2Error;

const AUTHORIZED_USERS: [&str; 3] = ["faultsmelts#0000", "ChrisW#6807", "Gio#6333"];

#[async_trait]
pub trait EcsCommands {
    /// Handles the ecs commands
    /// # Arguments
    /// * `message` - The message to parse
    /// * `ctx` - The context of the message
    /// * `msg` - The message
    async fn ec2_handler(&self, message: &str, ctx: &Context, msg: &Message);
    /// Starts the ec2 instance
    /// # Returns
    /// The status of the instance
    async fn start_instance(&self) -> Result<String, Ec2Error>;
    /// Stops the ec2 instance
    async fn stop_instance(&self) -> Result<(), Ec2Error>;
    /// Gets the status of the ec2 instance
    /// # Returns
    /// The status of the instance
    async fn get_instance_status(&self) -> Result<String, Ec2Error>;
    /// Gets the public ip of the ec2 instance
    /// # Returns
    /// The public ip of the instance
    async fn get_instance_ip(&self) -> Result<String, Ec2Error>;
    /// Prints the help message for the ecs commands
    /// # Returns
    /// The help message
    fn print_ecs_help(&self) -> String;
}

#[async_trait]
impl EcsCommands for Bot {
    async fn ec2_handler(&self, command: &str, ctx: &Context, msg: &Message) {
        match (
            command,
            AUTHORIZED_USERS.contains(&msg.author.tag().as_str()),
        ) {
            ("start", true) => start_instance(self, ctx, msg).await,
            ("stop", true) => stop_instance(self, ctx, msg).await,
            ("status", _) => get_status(self, ctx, msg).await,
            ("getip", _) => get_ip(self, ctx, msg).await,
            ("help", _) => {
                if let Err(e) = msg.channel_id.say(&ctx.http, self.print_ecs_help()).await {
                    error!("Error sending message: {:?}", e);
                }
            }
            (message, is_auth) => {
                if (message == "start" || message == "stop") && !is_auth {
                    if let Err(e) = msg
                        .channel_id
                        .say(&ctx.http, "You are not authorized to use this command.")
                        .await
                    {
                        error!("Error sending message: {:?}", e);
                    }
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

    async fn start_instance(&self) -> Result<String, Ec2Error> {
        let res = self
            .ec2_client
            .start_instances()
            .instance_ids(self.instance_id.clone())
            .send()
            .await
            .map_err(|e| Ec2Error::new(e.to_string()))?;
        Ok(res.starting_instances().unwrap()[0]
            .current_state()
            .unwrap()
            .name()
            .unwrap()
            .as_str()
            .to_string())
    }

    async fn stop_instance(&self) -> Result<(), Ec2Error> {
        self.ec2_client
            .stop_instances()
            .instance_ids(self.instance_id.clone())
            .send()
            .await
            .map_err(|e| Ec2Error::new(e.to_string()))?;
        Ok(())
    }

    async fn get_instance_status(&self) -> Result<String, Ec2Error> {
        let res = self
            .ec2_client
            .describe_instances()
            .instance_ids(self.instance_id.clone())
            .send()
            .await
            .map_err(|e| Ec2Error::new(e.to_string()))?;
        let status = res
            .reservations()
            .ok_or(Ec2Error::new("No instances found".into()))?[0]
            .instances()
            .ok_or(Ec2Error::new("No instances found".into()))?[0]
            .state()
            .ok_or(Ec2Error::new("No state found".into()))?
            .name()
            .ok_or(Ec2Error::new("No name found".into()))?
            .as_str()
            .to_string();
        Ok(status)
    }

    async fn get_instance_ip(&self) -> Result<String, Ec2Error> {
        let status = self.get_instance_status().await?;
        if status != "running" {
            return Err(Ec2Error::new("Instance is not running".into()));
        }
        Ok(self
            .ec2_client
            .describe_instances()
            .instance_ids(self.instance_id.clone())
            .send()
            .await
            .map_err(|e| Ec2Error::new(e.to_string()))?
            .reservations()
            .ok_or(Ec2Error::new("No reservations found".into()))?[0]
            .instances()
            .ok_or(Ec2Error::new("No instances found".into()))?[0]
            .public_ip_address()
            .ok_or(Ec2Error::new("No public ip found".into()))?
            .to_string())
    }

    fn print_ecs_help(&self) -> String {
        "
        $mc-start: Starts the Minecraft server (REQUIRES AUTHORIZATION)
        $mc-stop: Stops the Minecraft server (REQUIRES AUTHORIZATION)
        $mc-status: Gets the status of the Minecraft server
        $mc-getip: Gets the public ip of the Minecraft server
        $mc-help: Displays this message
        "
        .into()
    }
}

async fn get_ip(bot: &Bot, ctx: &Context, msg: &Message) {
    msg.channel_id
        .say(&ctx.http, "Getting instance ip...")
        .await
        .unwrap();
    let typing = msg.channel_id.start_typing(&ctx.http).unwrap();
    let ip = bot.get_instance_ip().await;
    if ip.is_err() {
        let ip = ip.unwrap_err();
        msg.channel_id
            .say(
                &ctx.http,
                format!("Error getting instance ip: {}", ip.message),
            )
            .await
            .unwrap();
        return;
    }
    let ip = ip.unwrap_or_else(|_| "undefined. Check Logs.".into());
    if let Err(e) = msg
        .channel_id
        .say(&ctx.http, format!("The ip of the instance is: {}", ip))
        .await
    {
        error!("Error sending message: {:?}", e);
    }
    if let Err(e) = typing.stop().ok_or("error stopping typing") {
        error!("Error stopping typing: {:?}", e);
    }
}

async fn get_status(bot: &Bot, ctx: &Context, msg: &Message) {
    let status = bot.get_instance_status().await;
    if status.is_err() {
        msg.channel_id
            .say(&ctx.http, "Error getting instance status")
            .await
            .unwrap();
        return;
    }
    let status = status.unwrap_or_else(|_| "undefined. Check Logs.".into());
    if let Err(e) = msg
        .channel_id
        .say(
            &ctx.http,
            format!("The status of the instance is: {}", status),
        )
        .await
    {
        error!("Error sending message: {:?}", e);
    }
}

async fn stop_instance(bot: &Bot, ctx: &Context, msg: &Message) {
    if let Err(e) = msg.channel_id.say(&ctx.http, "Stopping instance...").await {
        error!("Error sending message: {:?}", e);
    }
    let typing = msg.channel_id.start_typing(&ctx.http).unwrap();
    if bot.stop_instance().await.is_err() {
        msg.channel_id
            .say(&ctx.http, "Error stopping instance")
            .await
            .unwrap();
        return;
    }
    if let Err(e) = msg
        .channel_id
        .say(&ctx.http, "The instance has been stopped")
        .await
    {
        error!("Error sending message: {:?}", e);
    }
    typing.stop().unwrap();
}

async fn start_instance(bot: &Bot, ctx: &Context, msg: &Message) {
    if let Err(e) = msg.channel_id.say(&ctx.http, "Starting instance...").await {
        error!("Error sending message: {:?}", e);
    }
    let typing = msg.channel_id.start_typing(&ctx.http).unwrap();
    let status = bot.start_instance().await;
    if status.is_err() {
        msg.channel_id
            .say(&ctx.http, "Error starting instance")
            .await
            .unwrap();
        return;
    }
    let status = status.unwrap_or_else(|_| "undefined. Check Logs.".into());
    if let Err(e) = msg
        .channel_id
        .say(
            &ctx.http,
            format!("The status of the instance is now: {}", status),
        )
        .await
    {
        error!("Error sending message: {:?}", e);
    }
    // TODO: Get the public ip of the instance and send it to the user
    if let Err(e) = msg.channel_id.say(&ctx.http, "Getting public ip...").await {
        error!("Error sending message: {:?}", e);
    }
    loop {
        let status = bot.get_instance_status().await;
        if status.is_err() {
            msg.channel_id
                .say(&ctx.http, "Error getting instance status")
                .await
                .unwrap();
            return;
        }
        let status = status.unwrap_or_else(|_| "undefined. Check Logs.".into());
        if status == "running" {
            let ip = bot.get_instance_ip().await;
            if ip.is_err() {
                msg.channel_id
                    .say(&ctx.http, "Error getting instance ip")
                    .await
                    .unwrap();
                return;
            }
            let ip = ip.unwrap_or_else(|_| "undefined. Check Logs.".into());
            if let Err(e) = msg
                .channel_id
                .say(
                    &ctx.http,
                    format!("The public ip of the instance is: {}", ip),
                )
                .await
            {
                error!("Error sending message: {:?}", e);
            }
            break;
        }
        tokio::time::sleep(Duration::from_secs(3)).await;
    }
    typing.stop().unwrap();
}
