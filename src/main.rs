use animeboys_bot::{bot::Bot, wz};
use dotenv::dotenv;
use serenity::prelude::*;

/// To Cross compile for linux
/// cross build --release --target armv7-unknown-linux-gnueabihf
#[tokio::main]
async fn main() {
    dotenv().ok();
    // tracing_subscriber::fmt()
    //     .with_target(false)
    //     .with_max_level(tracing::Level::DEBUG)
    //     .compact()
    //     .init();
    // let token = std::env::var("DISCORD_TOKEN").expect("DISORD_TOKEN is required");
    // let instance_id = std::env::var("INSTANCE_ID").expect("INSTANCE_ID is required");

    // let intents = GatewayIntents::GUILD_MESSAGES
    //     | GatewayIntents::MESSAGE_CONTENT
    //     | GatewayIntents::GUILD_MEMBERS
    //     | GatewayIntents::GUILD_PRESENCES
    //     | GatewayIntents::GUILDS
    //     | GatewayIntents::DIRECT_MESSAGES;

    // let mut client = Client::builder(&token, intents)
    //     .event_handler(Bot::new(instance_id).await)
    //     .await
    //     .expect("Error creating client");

    // if let Err(why) = client.start().await {
    //     println!("Error starting client: {:?}", why);
    // }
    wz::get_all_loadouts().await;
}
