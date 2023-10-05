use dotenv::dotenv;
use serenity::prelude::*;

/// To Cross compile for linux
/// cross build --release --target armv7-unknown-linux-gnueabihf
#[tokio::main]
async fn main() {
    dotenv().ok();
    tracing_subscriber::fmt()
        .with_target(false)
        .with_max_level(tracing::Level::DEBUG)
        .compact()
        .init();
    let token = std::env::var("DISCORD_TOKEN").expect("DISORD_TOKEN is required");
    let instance_id = std::env::var("INSTANCE_ID").expect("INSTANCE_ID is required");
    let api_key = std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY is required");
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILD_MEMBERS
        | GatewayIntents::GUILD_PRESENCES
        | GatewayIntents::GUILDS
        | GatewayIntents::DIRECT_MESSAGES;

    let mut client = animeboys_bot::bot::create_bot(token, intents, api_key, instance_id).await;

    if let Err(why) = client.start().await {
        println!("Error starting client: {:?}", why);
    }
}
