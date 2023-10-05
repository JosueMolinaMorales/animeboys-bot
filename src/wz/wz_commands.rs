use anyhow::Context;
use serenity::{
    framework::standard::{
        macros::{command, group},
        Args, CommandResult,
    },
    model::prelude::Message,
    prelude::Context as Ctx,
};

use crate::wz::types::WzLoadouts;

use super::types::{TierListResponse, WZ_WEAPONS};
#[group("Warzone Commands")]
#[prefixes("wz")]
#[description("Commands for Warzone")]
#[summary("Commands for Warzone")]
#[commands(weapon_ids, ranked_build, top_3, future_features, help)]
struct WzCommands;

#[command]
#[description("Displays the help message")]
#[aliases("h")]
#[min_args(0)]
#[max_args(0)]
async fn help(ctx: &Ctx, msg: &Message) -> CommandResult {
    let help = "
    **Warzone Commands**
    `$wz weapon-ids` - Displays all weapon ids
    `$wz ranked-build <weapon_id>` - Displays the ranked build for the weapon
    `$wz top-3` - Displays the top 3 builds
    `$wz future-features` - Displays future features
    ";
    msg.channel_id.say(&ctx.http, help).await?;
    Ok(())
}

#[command("weapon-ids")]
#[description("Displays all weapon ids")]
#[aliases("wpids")]
#[min_args(0)]
#[max_args(0)]
async fn weapon_ids(ctx: &Ctx, msg: &Message) -> CommandResult {
    let mut weapons = String::new();
    for (key, value) in WZ_WEAPONS.entries() {
        weapons.push_str(&format!("{}: {}\n", key, value));
    }
    msg.channel_id.say(&ctx.http, weapons).await?;
    Ok(())
}

#[command("ranked-build")]
#[description("Displays the ranked build for the weapon")]
#[usage("<weapon_id>")]
#[example("kastov-762")]
#[aliases("rb")]
#[min_args(1)]
#[max_args(1)]
async fn ranked_build(ctx: &Ctx, msg: &Message, args: Args) -> CommandResult {
    let weapon_id = args.rest();
    let typing = msg.channel_id.start_typing(&ctx.http)?;
    let res = get_wz_ranked_build(weapon_id).await;
    msg.channel_id.say(&ctx.http, res?).await?;
    typing.stop().ok_or("Error stopping typing")?;
    Ok(())
}

#[command("top-3")]
#[description("Displays the top 3 builds")]
#[aliases("t3")]
#[min_args(0)]
#[max_args(0)]
async fn top_3(ctx: &Ctx, msg: &Message) -> CommandResult {
    let top_three_builds = get_top_three_builds().await;
    let mut top_three_builds = top_three_builds?
        .iter()
        .enumerate()
        .map(|(i, build)| format!("{}: {}", i + 1, build))
        .collect::<Vec<String>>()
        .join("\n");
    top_three_builds.insert_str(0, "Top 10 Builds:\n");
    msg.channel_id
        .send_message(&ctx.http, |m| m.content(top_three_builds))
        .await?;
    Ok(())
}

#[command("future-features")]
#[description("Displays future features")]
#[aliases("ff")]
#[min_args(0)]
#[max_args(0)]
async fn future_features(ctx: &Ctx, msg: &Message) -> CommandResult {
    let features = "
    **NOTE**: This Feature is Currently Under Testing. Expect issues.
            Future Features:
                - `$wz loadout <weapon_id>` - Displays the ranked loadout for the weapon
                - Better Querying for weapons: `$wz weapon <weapon_name>` or `$wz weapon <weapon_id>`
                - Displaying images of builds instead of text
    ";
    msg.channel_id.say(&ctx.http, features).await?;
    Ok(())
}

async fn get_wz_ranked_build(weapon_id: &str) -> Result<String, anyhow::Error> {
    let res = reqwest::Client::new()
        .get(format!(
            "https:///app.wzstats.gg/wz2/weapons/builds/wzstats/with-attachments/weapon/{}/?game=wz2",
            weapon_id
        ))
        .send()
        .await
        .context(format!("Failed to retrieve ranked build for {}", weapon_id))?;

    let loadouts = res.json::<WzLoadouts>().await.ok();
    if loadouts.is_none() {
        return Ok(format!(
            "No ranked build found for {}. Use `$wz weapon-ids` for a full list of weapon ids",
            weapon_id
        ));
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
    Ok(ranked_build.to_string())
}

pub async fn get_top_three_builds() -> Result<Vec<String>, anyhow::Error> {
    let res = reqwest::Client::new().get(
        "https://app.wzstats.gg/wz2/weapons/meta/weapons-and-tier-lists/?streamerProfileId=wzstats",
    ).send().await.context("Failed to retrieve top 3 builds")?;
    let res = res
        .json::<TierListResponse>()
        .await
        .context("Failed to parse top 3 builds")?;

    let tiers = res.wz_stats_tier_list.wz2_ranked;
    let top_10 = tiers.get_top_10();
    let mut top_3_builds = Vec::new();

    for weapon in top_10 {
        if top_3_builds.len() == 3 {
            break;
        }
        top_3_builds.push(get_wz_ranked_build(&weapon).await?);
    }

    Ok(top_3_builds)
}
