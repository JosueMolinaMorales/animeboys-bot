use std::{
    collections::HashMap,
    fmt::{Display, Formatter},
};

use phf::phf_map;
use serde::{Deserialize, Serialize};

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

#[derive(Debug, Serialize, Deserialize)]
pub struct VerticalTuning {
    pub value: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HorizontalTuning {
    pub value: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Attachment {
    pub attachment_id: String,
    pub vertical_tuning: serde_json::Value,
    pub horizontal_tuning: serde_json::Value,
    pub slot: String,
    pub name: String,
}
impl Display for Attachment {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut tunings = String::new();
        let (hz, vt) = self.get_tuning();
        tunings.push_str(&format!(":left_right_arrow: {} ", hz));
        tunings.push_str(&format!(":arrow_up_down: {}", vt));
        write!(f, "{}: {}", self.name, tunings)
    }
}
impl Attachment {
    pub fn get_tuning(&self) -> (f64, f64) {
        let mut hz = HorizontalTuning { value: 0.0 };
        let mut vt = VerticalTuning { value: 0.0 };

        if self.horizontal_tuning.is_f64() {
            hz.value = self.horizontal_tuning.as_f64().unwrap();
        } else if self.horizontal_tuning.is_object() {
            hz = serde_json::from_value(self.horizontal_tuning.clone())
                .expect("Error parsing horizontal tuning");
        }

        if self.vertical_tuning.is_f64() {
            vt.value = self.vertical_tuning.as_f64().unwrap();
        } else if self.vertical_tuning.is_object() {
            vt = serde_json::from_value(self.vertical_tuning.clone())
                .expect("Error parsing vertical tuning");
        }

        (hz.value, vt.value)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct Weapon {
    // WZ Stats Info
    pub id: String,
    pub cons: Vec<String>,
    pub pros: Vec<String>,
    pub r#type: String,
    pub title: String,
    pub added_at: Option<f64>,
    pub author_id: String,
    pub position: i32,
    pub weapon_id: String,
    pub created_at: String,
    pub playstyle: String,
    pub tier_score: Option<i32>,
    pub is_published: bool,
    pub updated_at: String,
    pub display_order: Option<i32>,
    pub interaction_count: Option<i32>,
    pub author_display_name: String,
    pub external_source_title: Option<String>,
    pub is_warzone_ranked_build: bool,
    pub external_source_image: Option<String>,
    pub is_ashika_build: bool,
    pub pros_cons: serde_json::Value,
    pub stats_analysis: serde_json::Value,

    // Weapon Info
    pub ads_time: Option<String>,
    pub description: String,
    pub movement_speed: Option<String>,
    pub bullet_velocity: Option<String>,
    pub ads_movement_speed: Option<String>,
    pub vertical_recoil_reduction: Option<String>,
    pub horizontal_recoil_reduction: Option<String>,

    // Attachments
    pub stock: Option<Attachment>,
    pub optic: Option<Attachment>,
    pub ammunition: Option<Attachment>,
    pub muzzle: Option<Attachment>,
    pub laser: Option<Attachment>,
    pub barrel: Option<Attachment>,
    pub magazine: Option<Attachment>,
    pub rear_grip: Option<Attachment>,
    pub underbarrel: Option<Attachment>,
    pub guard: Option<Attachment>,
    pub comb: Option<Attachment>,
    pub rail: Option<Attachment>,
    pub bolt: Option<Attachment>,
    pub trigger_action: Option<Attachment>,
}

impl Weapon {
    pub fn get_weapon_name(&self) -> String {
        self.weapon_id
            .split('-')
            .map(|s| s.to_uppercase())
            .collect::<Vec<String>>()
            .join(" ")
    }

    pub fn get_loadout_attachments(&self) -> HashMap<String, Attachment> {
        // Go through attachements and get the name of each attachment
        let mut map = HashMap::new();
        // Muzzle
        if let Some(muzzle) = &self.muzzle {
            map.insert("Muzzle".to_string(), muzzle.to_owned());
        }
        // Barrel
        if let Some(barrel) = &self.barrel {
            map.insert("Barrel".to_string(), barrel.to_owned());
        }
        // Laser
        if let Some(laser) = &self.laser {
            map.insert("Laser".to_string(), laser.to_owned());
        }
        // Optic
        if let Some(optic) = &self.optic {
            map.insert("Optic".to_string(), optic.to_owned());
        }
        // Stock
        if let Some(stock) = &self.stock {
            map.insert("Stock".to_string(), stock.to_owned());
        }
        // Underbarrel
        if let Some(underbarrel) = &self.underbarrel {
            map.insert("Underbarrel".to_string(), underbarrel.to_owned());
        }
        // Ammunition
        if let Some(ammunition) = &self.ammunition {
            map.insert("Ammunition".to_string(), ammunition.to_owned());
        }
        // Rear Grip
        if let Some(rear_grip) = &self.rear_grip {
            map.insert("Rear Grip".to_string(), rear_grip.to_owned());
        }
        // Magazine
        if let Some(mag) = &self.magazine {
            map.insert("Magazine".to_string(), mag.to_owned());
        }

        map
    }

    pub fn to_string(&self) -> String {
        let mut loadout_string = String::new();
        loadout_string.push_str(&format!("Weapon: {}\n", self.get_weapon_name()));
        loadout_string.push_str(&format!("Playstyle: {}\n", self.playstyle));
        loadout_string.push_str(&format!("Description: {}\n", self.description));
        loadout_string.push_str(&format!("Attachments:\n"));
        for (key, value) in self.get_loadout_attachments() {
            loadout_string.push_str(&format!("    {} - {}\n", key, value));
        }
        loadout_string
    }
}

impl Default for Weapon {
    fn default() -> Self {
        Self {
            id: Default::default(),
            cons: Default::default(),
            pros: Default::default(),
            r#type: Default::default(),
            title: Default::default(),
            muzzle: Default::default(),
            added_at: Default::default(),
            trigger_action: Default::default(),
            optic: Default::default(),
            ammunition: Default::default(),
            ads_time: Default::default(),
            author_id: Default::default(),
            guard: Default::default(),
            comb: Default::default(),
            stock: Default::default(),
            magazine: Default::default(),
            laser: Default::default(),
            stats_analysis: Default::default(),
            barrel: Default::default(),
            is_ashika_build: Default::default(),
            position: Default::default(),
            rail: Default::default(),
            bolt: Default::default(),
            rear_grip: Default::default(),
            weapon_id: Default::default(),
            created_at: Default::default(),
            playstyle: Default::default(),
            tier_score: Default::default(),
            updated_at: Default::default(),
            pros_cons: Default::default(),
            description: Default::default(),
            is_published: Default::default(),
            underbarrel: Default::default(),
            display_order: Default::default(),
            movement_speed: Default::default(),
            bullet_velocity: Default::default(),
            ads_movement_speed: Default::default(),
            interaction_count: Default::default(),
            author_display_name: Default::default(),
            vertical_recoil_reduction: Default::default(),
            horizontal_recoil_reduction: Default::default(),
            external_source_title: Default::default(),
            is_warzone_ranked_build: Default::default(),
            external_source_image: Default::default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WzLoadouts {
    pub builds: Vec<Weapon>,
}

impl Default for WzLoadouts {
    fn default() -> Self {
        Self { builds: vec![] }
    }
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub enum WzWeaponName {
    FiftyGS,
    FiveFiveSixIcarus,
    NineMilliMeterDaemon,
    BasP,
    Basilisk,
    Bryson800,
    Bryson890,
    Carrack300,
    Chimera,
    CronenSquall,
    Ebr14,
    Expedite12,
    Fennec45,
    FjxImperium,
    FrAvancer,
    FssHurricane,
    FtacRecon,
    FtacSiege,
    GsMagma,
    Hcr56,
    Iso45,
    Iso9mm,
    IsoHemlock,
    Kastov545,
    Kastov74u,
    Kastov762,
    KvBroadside,
    LaB330,
    Lachmann556,
    Lachmann762,
    LachmannShroud,
    LachmannSub,
    LmS,
    Lockwood300,
    LockwoodMk2,
    M13b,
    M13c,
    M16,
    M4,
    Mcpr300,
    Minibak,
    MxGuardian,
    Mx9,
    P890,
    Pdsw528,
    RaalMg,
    RappH,
    Rpk,
    SaB50,
    SakinMg38,
    Signal50,
    So14,
    SpR208,
    SpX80,
    Stb556,
    Taq56,
    TaqM,
    TaqV,
    TempusRazorback,
    TempusTorrent,
    Tr76Geist,
    Vaznev9k,
    Vel46,
    VictusXmr,
    X12,
    X13Auto,
}

impl Display for WzWeaponName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            WzWeaponName::FiftyGS => "50 Gs",
            WzWeaponName::FiveFiveSixIcarus => "556 Icarus",
            WzWeaponName::NineMilliMeterDaemon => "9mm Daemon",
            WzWeaponName::BasP => "Bas P",
            WzWeaponName::Basilisk => "Basilisk",
            WzWeaponName::Bryson800 => "Bryson 800",
            WzWeaponName::Bryson890 => "Bryson 890",
            WzWeaponName::Carrack300 => "Carrack 300",
            WzWeaponName::Chimera => "Chimera",
            WzWeaponName::CronenSquall => "Cronen Squall",
            WzWeaponName::Ebr14 => "Ebr 14",
            WzWeaponName::Expedite12 => "Expedite 12",
            WzWeaponName::Fennec45 => "Fennec 45",
            WzWeaponName::FjxImperium => "Fjx Imperium",
            WzWeaponName::FrAvancer => "Fr Avancer",
            WzWeaponName::FssHurricane => "Fss Hurricane",
            WzWeaponName::FtacRecon => "Ftac Recon",
            WzWeaponName::FtacSiege => "Ftac Siege",
            WzWeaponName::GsMagma => "Gs Magma",
            WzWeaponName::Hcr56 => "Hcr 56",
            WzWeaponName::Iso45 => "Iso 45",
            WzWeaponName::Iso9mm => "Iso 9mm",
            WzWeaponName::IsoHemlock => "Iso Hemlock",
            WzWeaponName::Kastov545 => "Kastov 545",
            WzWeaponName::Kastov74u => "Kastov 74u",
            WzWeaponName::Kastov762 => "Kastov 762",
            WzWeaponName::KvBroadside => "Kv Broadside",
            WzWeaponName::LaB330 => "La B 330",
            WzWeaponName::Lachmann556 => "Lachmann 556",
            WzWeaponName::Lachmann762 => "Lachmann 762",
            WzWeaponName::LachmannShroud => "Lachmann Shroud",
            WzWeaponName::LachmannSub => "Lachmann Sub",
            WzWeaponName::LmS => "Lm S",
            WzWeaponName::Lockwood300 => "Lockwood 300",
            WzWeaponName::LockwoodMk2 => "Lockwood Mk2",
            WzWeaponName::M13b => "M13b",
            WzWeaponName::M13c => "M13c",
            WzWeaponName::M16 => "M16",
            WzWeaponName::M4 => "M4",
            WzWeaponName::Mcpr300 => "Mcpr 300",
            WzWeaponName::Minibak => "Minibak",
            WzWeaponName::MxGuardian => "Mx Guardian",
            WzWeaponName::Mx9 => "Mx9",
            WzWeaponName::P890 => "P890",
            WzWeaponName::Pdsw528 => "Pdsw 528",
            WzWeaponName::RaalMg => "Raal Mg",
            WzWeaponName::RappH => "Rapp H",
            WzWeaponName::Rpk => "Rpk",
            WzWeaponName::SaB50 => "Sa B 50",
            WzWeaponName::SakinMg38 => "Sakin Mg38",
            WzWeaponName::Signal50 => "Signal 50",
            WzWeaponName::So14 => "So 14",
            WzWeaponName::SpR208 => "Sp R 208",
            WzWeaponName::SpX80 => "Sp X 80",
            WzWeaponName::Stb556 => "Stb 556",
            WzWeaponName::Taq56 => "Taq 56",
            WzWeaponName::TaqM => "Taq M",
            WzWeaponName::TaqV => "Taq V",
            WzWeaponName::TempusRazorback => "Tempus Razorback",
            WzWeaponName::TempusTorrent => "Tempus Torrent",
            WzWeaponName::Tr76Geist => "Tr 76 Geist",
            WzWeaponName::Vaznev9k => "Vaznev 9k",
            WzWeaponName::Vel46 => "Vel 46",
            WzWeaponName::VictusXmr => "Victus Xmr",
            WzWeaponName::X12 => "X12",
            WzWeaponName::X13Auto => "X13 Auto",
        };
        write!(f, "{}", name)
    }
}

pub enum Playstyles {
    AdsSpeed,
    BulletVelocity,
    FireRate,
    HipFire,
    LongRange,
    LowRecoil,
    Mobility,
    ShortRange,
    SniperSupport,
}

impl Display for Playstyles {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Playstyles::AdsSpeed => write!(f, "Ads Speed"),
            Playstyles::BulletVelocity => write!(f, "Bullet Velocity"),
            Playstyles::FireRate => write!(f, "Fire Rate"),
            Playstyles::HipFire => write!(f, "Hip Fire"),
            Playstyles::LongRange => write!(f, "Long Range"),
            Playstyles::LowRecoil => write!(f, "Low Recoil"),
            Playstyles::Mobility => write!(f, "Mobility"),
            Playstyles::ShortRange => write!(f, "Short Range"),
            Playstyles::SniperSupport => write!(f, "Sniper Support"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct TierListResponse {
    pub wz_stats_tier_list: WzStatsTierList,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct WzStatsTierList {
    pub streamer_profile_id: String,
    pub ashika_island: Tiers,
    pub al_mazrah: Tiers,
    pub mw2_ranked: Tiers,
    pub wz2_ranked: Tiers,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "UPPERCASE")]
#[serde(default)]
pub struct Tiers {
    pub meta: Vec<String>,
    pub a: Vec<String>,
    pub b: Vec<String>,
    pub c: Vec<String>,
    pub d: Vec<String>,
}

impl Tiers {
    pub fn get_top_10(&self) -> Vec<String> {
        let mut top_ten_weapons: Vec<String> = vec![];
        let mut get_weapons = |weapons: &Vec<String>| {
            for weapon in weapons {
                if top_ten_weapons.len() >= 10 {
                    break;
                }
                top_ten_weapons.push(weapon.to_owned())
            }
        };
        get_weapons(&self.meta);
        get_weapons(&self.a);
        get_weapons(&self.b);
        get_weapons(&self.c);
        get_weapons(&self.d);

        top_ten_weapons
    }
}
