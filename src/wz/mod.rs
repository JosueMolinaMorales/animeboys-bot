use std::{
    collections::HashMap,
    fmt::{Display, Formatter},
};

use phf::phf_map;
use serde::{Deserialize, Serialize};
use serenity::{async_trait, model::prelude::Message, prelude::Context};
use tracing::error;

use crate::bot::Bot;

use self::types::{WzLoadouts, WzWeaponName};

pub mod types;

/// Current as of Warzone Season 5 (9/24/2023)
pub static WZ_WEAPONS: phf::Map<&str, WzWeaponName> = phf_map! {
    "50gs" => WzWeaponName::FiftyGS,
    "556-icarus" => WzWeaponName::FiveFiveSixIcarus,
    "9mm-daemon" => WzWeaponName::NineMilliMeterDaemon,
    "bas-p" => WzWeaponName::BasP,
    "basilisk" => WzWeaponName::Basilisk,
    "bryson-800" => WzWeaponName::Bryson800,
    "bryson-890" => WzWeaponName::Bryson890,
    "carrack-300" => WzWeaponName::Carrack300,
    "chimera" => WzWeaponName::Chimera,
    "cronen-squall" => WzWeaponName::CronenSquall,
    "ebr-14" => WzWeaponName::Ebr14,
    "expedite-12" => WzWeaponName::Expedite12,
    "fennec-45" => WzWeaponName::Fennec45,
    "fjx-imperium" => WzWeaponName::FjxImperium,
    "fr-avancer" => WzWeaponName::FrAvancer,
    "fss-hurricane" => WzWeaponName::FssHurricane,
    "ftac-recon" => WzWeaponName::FtacRecon,
    "ftac-siege" => WzWeaponName::FtacSiege,
    "gs-magma" => WzWeaponName::GsMagma,
    "hcr-56" => WzWeaponName::Hcr56,
    "iso-45" => WzWeaponName::Iso45,
    "iso-9mm" => WzWeaponName::Iso9mm,
    "iso-hemlock" => WzWeaponName::IsoHemlock,
    "kastov-545" => WzWeaponName::Kastov545,
    "kastov-74u" => WzWeaponName::Kastov74u,
    "kastov-762" => WzWeaponName::Kastov762,
    "kv-broadside" => WzWeaponName::KvBroadside,
    "la-b-330" => WzWeaponName::LaB330,
    "lachmann-556" => WzWeaponName::Lachmann556,
    "lachmann-762" => WzWeaponName::Lachmann762,
    "lachmann-shroud" => WzWeaponName::LachmannShroud,
    "lachmann-sub" => WzWeaponName::LachmannSub,
    "lm-s" => WzWeaponName::LmS,
    "lockwood-300" => WzWeaponName::Lockwood300,
    "lockwood-mk2" => WzWeaponName::LockwoodMk2,
    "m13b" => WzWeaponName::M13b,
    "m13c" => WzWeaponName::M13c,
    "m16" => WzWeaponName::M16,
    "m4" => WzWeaponName::M4,
    "mcpr-300" => WzWeaponName::Mcpr300,
    "minibak" => WzWeaponName::Minibak,
    "mx-guardian" => WzWeaponName::MxGuardian,
    "mx9" => WzWeaponName::Mx9,
    "p890" => WzWeaponName::P890,
    "pdsw-528" => WzWeaponName::Pdsw528,
    "raal-mg" => WzWeaponName::RaalMg
};

#[async_trait]
pub trait WzLoadoutCommands {
    async fn wz_loadout_handler(&self, command: Vec<&str>, ctx: &Context, msg: &Message);
    async fn get_ranked_build(&self);
    async fn display_weapon_ids(&self) -> String;
    async fn print_wz_loadout_help(&self) -> String;
}

#[async_trait]
impl WzLoadoutCommands for Bot {
    async fn wz_loadout_handler(&self, command: Vec<&str>, ctx: &Context, msg: &Message) {
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
        `$wz help` - Displays this message
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

async fn get_top_ten_builds() -> Vec<String> {
    let res = reqwest::Client::new().get(
        "https://app.wzstats.gg/wz2/weapons/meta/weapons-and-tier-lists/?streamerProfileId=wzstats",
    ).send().await.unwrap();
}

pub async fn get_all_loadouts() {
    let res = reqwest::Client::new()
        .get("https://app.wzstats.gg/wz2/weapons/builds/wzstats/with-attachments/?game=wz2")
        .send()
        .await
        .unwrap();

    let mut loadouts: WzLoadouts = res.json().await.unwrap();

    // Sort by interaction count
    loadouts
        .builds
        .sort_by(|a, b| a.interaction_count.cmp(&b.interaction_count));
    // Get the top ten builds
    let builds = loadouts.builds[0..5].to_vec();

    println!("{:#?}", builds);
}

pub async fn get_kastov_loadouts() {
    // let res = reqwest::Client::new()
    //     .get("https://app.wzstats.gg/wz2/weapons/builds/wzstats/with-attachments/?game=wz2")
    //     .send()
    //     .await
    //     .unwrap();
    let res = reqwest::Client::new()
        .get("https:///app.wzstats.gg/wz2/weapons/builds/wzstats/with-attachments/weapon/m13b/?game=wz2")
        .send()
        .await
        .unwrap();

    let mut loadouts: WzLoadouts = res.json().await.unwrap();
    // println!("{:?}", loadouts);
    for weapon in loadouts.builds.iter() {
        println!("{:#?}", weapon);
        weapon.get_loadout_attachments();
    }

    // Sort builds by position
    loadouts.builds.sort_by(|a, b| a.position.cmp(&b.position));

    println!("{:#?}", loadouts.builds);
    // Get all unique weapon ids
    // let mut weapon_ids: Vec<String> = loadouts
    //     .builds
    //     .iter()
    //     .map(|w| w.weapon_id.clone())
    //     .collect();

    // // Remove duplicates
    // weapon_ids.sort();
    // weapon_ids.dedup();

    // println!("{:?}", weapon_ids);
}
