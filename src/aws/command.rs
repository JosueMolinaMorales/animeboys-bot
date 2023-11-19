use std::time::Duration;

use serenity::{
    framework::standard::{
        macros::{check, command, group},
        Args, CommandOptions, CommandResult, Reason,
    },
    model::channel::Message,
    prelude::Context,
};

use crate::aws::ec2::Ec2Client;

const AUTHORIZED_USERS: [&str; 3] = ["faultsmelts#0000", "chrisw6807#0000", "Gio#6333"];

#[group("Minecraft Commands")]
#[prefixes("minecraft", "mc")]
#[description("Commands for managing the minecraft server")]
#[summary("Commands for managing the minecraft server")]
#[commands(start, stop, status, getip, help)]
struct MinecraftCommands;

#[command]
#[description("Displays the help message")]
#[aliases("h")]
#[min_args(0)]
#[max_args(0)]
async fn help(ctx: &Context, msg: &Message) -> CommandResult {
    let help = "
    **Minecraft Commands**
    `$mc start` - Starts the minecraft server
    `$mc stop` - Stops the minecraft server
    `$mc status` - Displays the status of the minecraft server
    `$mc getip` - Displays the public ip of the minecraft server
    ";
    msg.channel_id.say(&ctx.http, help).await?;
    Ok(())
}

#[command]
#[description("Starts the minecraft server")]
#[usage("start")]
#[example("start")]
#[min_args(0)]
#[max_args(0)]
#[checks(MinecraftAdmin)]
async fn start(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id
        .say(&ctx.http, "Starting instance...")
        .await?;
    let typing = msg.channel_id.start_typing(&ctx.http)?;

    // Get ec2 client
    let data = ctx.data.read().await;
    let ec2_client = data
        .get::<Ec2Client>()
        .ok_or("Ec2Client not found in context")?;

    let status = match ec2_client.start_instance().await {
        Ok(status) => status,
        Err(e) => {
            msg.channel_id
                .say(&ctx.http, format!("Error starting instance: {}", e.message))
                .await?;
            return Ok(());
        }
    };

    msg.channel_id
        .say(
            &ctx.http,
            format!("The status of the instance is now: {}", status),
        )
        .await?;

    msg.channel_id
        .say(&ctx.http, "Getting public ip...")
        .await?;

    loop {
        let status = match ec2_client.get_instance_status().await {
            Ok(status) => status,
            Err(e) => {
                msg.channel_id
                    .say(
                        &ctx.http,
                        format!("Error getting instance status: {}", e.message),
                    )
                    .await?;
                return Ok(());
            }
        };

        if status == "running" {
            let ip = match ec2_client.get_instance_ip().await {
                Ok(ip) => ip,
                Err(e) => {
                    msg.channel_id
                        .say(
                            &ctx.http,
                            format!("Error getting instance ip: {}", e.message),
                        )
                        .await?;
                    return Ok(());
                }
            };

            msg.channel_id
                .say(
                    &ctx.http,
                    format!("The public ip of the instance is: {}", ip),
                )
                .await?;
            break;
        }
        tokio::time::sleep(Duration::from_secs(3)).await;
    }

    typing.stop().ok_or("error stopping typing")?;
    Ok(())
}

#[command]
#[description("Stops the minecraft server")]
#[usage("stop")]
#[example("stop")]
#[min_args(0)]
#[max_args(0)]
#[checks(MinecraftAdmin)]
async fn stop(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id
        .say(&ctx.http, "Stopping instance...")
        .await?;
    let typing = msg.channel_id.start_typing(&ctx.http)?;

    let data = ctx.data.read().await;
    let ec2_client = data
        .get::<Ec2Client>()
        .ok_or("Ec2Client not found in context")?;

    match ec2_client.stop_instance().await {
        Ok(_) => {
            msg.channel_id
                .say(&ctx.http, "The instance has been stopped")
                .await?;
        }
        Err(e) => {
            msg.channel_id
                .say(&ctx.http, format!("Error stopping instance: {}", e.message))
                .await?;
        }
    }

    typing.stop().unwrap();
    Ok(())
}

#[command]
#[description("Gets the status of the minecraft server")]
async fn status(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;

    let ec2_client = data
        .get::<Ec2Client>()
        .ok_or("Ec2Client not found in context")?;

    let status = match ec2_client.get_instance_status().await {
        Ok(status) => status,
        Err(e) => {
            msg.channel_id
                .say(
                    &ctx.http,
                    format!("Error getting instance status: {}", e.message),
                )
                .await
                .unwrap();
            return Ok(());
        }
    };
    msg.channel_id
        .say(
            &ctx.http,
            format!("The status of the instance is: {}", status),
        )
        .await?;
    Ok(())
}

#[command("getip")]
#[description("Gets the public ip of the minecraft server")]
#[aliases("ip")]
async fn getip(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    msg.channel_id
        .say(&ctx.http, "Getting instance ip...")
        .await?;
    let typing = msg.channel_id.start_typing(&ctx.http)?;

    let data = ctx.data.read().await;
    let ec2_client = data
        .get::<Ec2Client>()
        .ok_or("Ec2Client not found in context")?;

    let ip = match ec2_client.get_instance_ip().await {
        Ok(ip) => ip,
        Err(e) => {
            msg.channel_id
                .say(
                    &ctx.http,
                    format!("Error getting instance ip: {}", e.message),
                )
                .await?;
            return Ok(());
        }
    };

    msg.channel_id
        .say(&ctx.http, format!("The ip of the instance is: {}", ip))
        .await?;

    typing.stop().ok_or("error stopping typing")?;

    Ok(())
}

#[check]
#[name = "MinecraftAdmin"]
async fn has_minecraft_access(
    ctx: &Context,
    msg: &Message,
    _: &mut Args,
    _: &CommandOptions,
) -> Result<(), Reason> {
    // Check that the user's username is in the AUTHORIZED_USERS array
    if !AUTHORIZED_USERS.contains(&msg.author.tag().as_str()) {
        msg.reply(ctx, "You are not authorized to use this command")
            .await
            .unwrap();
        return Err(Reason::User(
            "You are not authorized to use this command".into(),
        ));
    }

    Ok(())
}
