use serenity::{async_trait, model::prelude::Message, prelude::Context};
use tracing::error;

use crate::{bot::Bot, wz::types::WzLoadouts};

use super::types::{TierListResponse, WZ_WEAPONS};

#[async_trait]
pub trait WzLoadoutCommands {
    async fn wz_loadout_handler(&self, command: Vec<&str>, ctx: &Context, msg: &Message);
    async fn get_ranked_build(&self);
    async fn display_weapon_ids(&self) -> String;
    async fn print_wz_loadout_help(&self) -> String;
    async fn print_wz_future_features(&self) -> String;
}

#[async_trait]
impl WzLoadoutCommands for Bot {
    async fn wz_loadout_handler(&self, command: Vec<&str>, ctx: &Context, msg: &Message) {
        // Start typing to show the user that the bot is working
        let typing = msg.channel_id.start_typing(&ctx.http).unwrap();
        match command.get(0) {
            Some(&"weapon-ids") => {
                if let Err(e) = msg
                    .channel_id
                    .say(&ctx.http, self.display_weapon_ids().await)
                    .await
                {
                    error!("Error sending message: {:?}", e);
                }
            }
            Some(&"ranked-build") => {
                // Check if the user provided a weapon id
                let weapon_id = match command.get(1) {
                    Some(weapon_id) => weapon_id,
                    None => {
                        if let Err(e) = msg
                            .channel_id
                            .say(&ctx.http, "Please provide a weapon id.")
                            .await
                        {
                            error!("Error sending message: {:?}", e);
                        }
                        return;
                    }
                };
                if let Err(e) = msg
                    .channel_id
                    .say(&ctx.http, get_wz_ranked_build(weapon_id).await)
                    .await
                {
                    error!("Error sending message: {:?}", e);
                }
            }
            Some(&"top-3") => {
                let top_three_builds = get_top_three_builds().await;
                let mut top_three_builds = top_three_builds
                    .iter()
                    .enumerate()
                    .map(|(i, build)| format!("{}: {}", i + 1, build))
                    .collect::<Vec<String>>()
                    .join("\n");
                top_three_builds.insert_str(0, "Top 10 Builds:\n");
                if let Err(e) = msg
                    .channel_id
                    .send_message(&ctx.http, |m| m.content(top_three_builds))
                    .await
                {
                    error!("Error sending message: {:?}", e);
                }
            }
            Some(&"future-features") => {
                if let Err(e) = msg
                    .channel_id
                    .say(&ctx.http, self.print_wz_future_features().await)
                    .await
                {
                    error!("Error sending message: {:?}", e);
                }
            }
            Some(&"help") => {
                if let Err(e) = msg
                    .channel_id
                    .say(&ctx.http, self.print_wz_loadout_help().await)
                    .await
                {
                    error!("Error sending message: {:?}", e);
                }
            }
            _ => {
                if let Err(e) = msg
                    .channel_id
                    .say(&ctx.http, "Invalid command. Use $wz-help for help.")
                    .await
                {
                    error!("Error sending message: {:?}", e);
                }
            }
        }
        // Stop typing
        typing.stop().unwrap();
    }

    async fn get_ranked_build(&self) {}

    async fn display_weapon_ids(&self) -> String {
        let mut weapons = String::new();
        for (key, value) in WZ_WEAPONS.entries() {
            weapons.push_str(&format!("{}: {}\n", key, value));
        }
        weapons
    }

    async fn print_wz_loadout_help(&self) -> String {
        "
**NOTE**: This Feature is Currently Under Testing. Expect issues.
    The following commands are available:
        `$wz weapon-ids` - Displays all weapon ids
        `$wz ranked-build <weapon_id>` - Displays the ranked build for the weapon
        `$wz top-3` - Displays the top 10 builds
        `$wz help` - Displays this message
        `$wz future-features` - Displays future features
        "
        .into()
    }

    async fn print_wz_future_features(&self) -> String {
        "
**NOTE**: This Feature is Currently Under Testing. Expect issues.
        Future Features:
            - `$wz loadout <weapon_id>` - Displays the ranked loadout for the weapon
            - Better Querying for weapons: `$wz weapon <weapon_name>` or `$wz weapon <weapon_id>`
            - Displaying images of builds instead of text
"
        .into()
    }
}

async fn get_wz_ranked_build(weapon_id: &str) -> String {
    let res = reqwest::Client::new()
        .get(format!(
            "https:///app.wzstats.gg/wz2/weapons/builds/wzstats/with-attachments/weapon/{}/?game=wz2",
            weapon_id
        ))
        .send()
        .await
        .unwrap(); // TODO: Handle error

    let loadouts = res.json::<WzLoadouts>().await.ok();
    if loadouts.is_none() {
        return format!(
            "No ranked build found for {}. Use `$wz weapon-ids` for a full list of weapon ids",
            weapon_id
        );
    }

    let mut loadouts = loadouts.unwrap();
    // Find the ranked build
    let ranked_build = loadouts.builds.iter().find(|w| w.is_warzone_ranked_build);
    // If this weapon doesn't have a ranked build, return the first position build
    let ranked_build = match ranked_build {
        Some(ranked_build) => ranked_build,
        None => {
            loadouts.builds.sort_by(|a, b| a.position.cmp(&b.position));
            &loadouts.builds[0]
        }
    };

    // Return String of ranked build
    ranked_build.to_string()
}

pub async fn get_top_three_builds() -> Vec<String> {
    let res = reqwest::Client::new().get(
        "https://app.wzstats.gg/wz2/weapons/meta/weapons-and-tier-lists/?streamerProfileId=wzstats",
    ).send().await.unwrap();
    let res = res.json::<TierListResponse>().await.unwrap();

    let tiers = res.wz_stats_tier_list.wz2_ranked;
    let top_10 = tiers.get_top_10();
    let mut top_3_builds = Vec::new();

    for weapon in top_10 {
        if top_3_builds.len() == 3 {
            break;
        }
        top_3_builds.push(get_wz_ranked_build(&weapon).await);
    }

    top_3_builds
}
