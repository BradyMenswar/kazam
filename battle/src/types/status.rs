//! Status conditions (volatile and non-volatile)

/// Non-volatile status conditions (persist through switching)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Status {
    Burn,
    Freeze,
    Paralysis,
    Poison,
    BadPoison, // Toxic
    Sleep,
}

impl Status {
    /// Parse from protocol string ("brn", "frz", "par", "psn", "tox", "slp")
    pub fn from_protocol(s: &str) -> Option<Self> {
        match s {
            "brn" => Some(Status::Burn),
            "frz" => Some(Status::Freeze),
            "par" => Some(Status::Paralysis),
            "psn" => Some(Status::Poison),
            "tox" => Some(Status::BadPoison),
            "slp" => Some(Status::Sleep),
            _ => None,
        }
    }

    /// Convert to protocol format
    pub fn to_protocol(&self) -> &'static str {
        match self {
            Status::Burn => "brn",
            Status::Freeze => "frz",
            Status::Paralysis => "par",
            Status::Poison => "psn",
            Status::BadPoison => "tox",
            Status::Sleep => "slp",
        }
    }

    /// Get display name
    pub fn as_str(&self) -> &'static str {
        match self {
            Status::Burn => "Burn",
            Status::Freeze => "Freeze",
            Status::Paralysis => "Paralysis",
            Status::Poison => "Poison",
            Status::BadPoison => "Toxic",
            Status::Sleep => "Sleep",
        }
    }
}

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Volatile status conditions (cleared on switching)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Volatile {
    // Movement restriction
    Trapped,     // Mean Look, Spider Web, Block
    PartialTrap, // Bind, Wrap, Fire Spin, etc.

    // Mental effects
    Confusion,
    Taunt,
    Encore,
    Disable,
    Torment,
    Infatuation, // Attract

    // Stat/combat related
    FocusEnergy,
    LaserFocus,

    // Damage over time / healing
    LeechSeed,
    Curse, // Ghost-type curse
    PerishSong,
    Nightmare,

    // Protection
    Protect,
    Endure,
    Substitute,

    // Semi-invulnerable
    Fly,
    Dig,
    Dive,
    ShadowForce,
    PhantomForce,
    Bounce,
    SkyDrop,

    // Turn-based states
    Flinch,
    Yawn,       // About to fall asleep
    Recharging, // Hyper Beam, etc.
    Charging,   // Solar Beam, etc.

    // Multi-turn moves
    Bide,
    Uproar,
    Thrash, // Outrage, Petal Dance, etc.
    Rollout,

    // Type/immunity related
    MagnetRise,
    Telekinesis,
    Smackdown, // Grounded
    Ingrain,
    AquaRing,

    // Ability-related
    FlashFire,   // Flash Fire activated
    SlowStart,   // Regigigas ability counter
    Truant,      // Truant turn tracking
    Unburden,    // Speed boost after item loss
    GastroAcid,  // Ability suppressed
    Imprison,    // Moves locked
    Minimize,    // Stomp/etc does double damage
    DefenseCurl, // Rollout does double damage

    // Transform
    Transformed,

    // Misc
    Roost,        // Lost Flying type this turn
    Stockpile,    // 1-3 layers
    HelpingHand,  // Power boost from ally
    PowerTrick,   // Atk/Def swapped
    Autotomize,   // Weight reduced
    MagicCoat,    // Reflecting moves
    Snatch,       // Stealing moves
    DestinyBond,  // Taking opponent down
    Grudge,       // PP drain on KO
    Rage,         // Attack boost on hit
    FocusPunch,   // Charging Focus Punch
    MudSport,     // Electric weakened (old gens)
    WaterSport,   // Fire weakened (old gens)
    Electrify,    // Next move becomes Electric
    CenterOfAttention, // Follow Me/Rage Powder

    // Gen 8+
    Dynamaxed,
    Octolock,
    TarShot,
    NoRetreat,

    // Gen 9+
    Terastallized,
    SaltCure,
    Syrupy,

    /// Unknown volatile from protocol
    Other(String),
}

impl Volatile {
    /// Parse from protocol string
    pub fn from_protocol(s: &str) -> Self {
        // Strip common prefixes
        let clean = s
            .strip_prefix("move: ")
            .or_else(|| s.strip_prefix("ability: "))
            .unwrap_or(s);

        // Normalize: lowercase and remove spaces, dashes, apostrophes
        let normalized = clean.to_lowercase().replace([' ', '-', '\''], "");

        match normalized.as_str() {
            "trapped" | "meanloop" | "spiderweb" | "block" => Volatile::Trapped,
            "partialtrap" | "bind" | "wrap" | "firespin" | "clamp" | "whirlpool" | "sandtomb"
            | "magmastorm" | "infestation" | "snaptrip" => Volatile::PartialTrap,

            "confusion" | "confused" => Volatile::Confusion,
            "taunt" => Volatile::Taunt,
            "encore" => Volatile::Encore,
            "disable" | "disabled" => Volatile::Disable,
            "torment" => Volatile::Torment,
            "attract" | "infatuation" => Volatile::Infatuation,

            "focusenergy" => Volatile::FocusEnergy,
            "laserfocus" => Volatile::LaserFocus,

            "leechseed" => Volatile::LeechSeed,
            "curse" => Volatile::Curse,
            "perishsong" | "perish3" | "perish2" | "perish1" => Volatile::PerishSong,
            "nightmare" => Volatile::Nightmare,

            "protect" | "detect" | "kingsshield" | "spikyshield" | "banefulbunker"
            | "obstruct" | "silktrap" | "burningbulwark" => Volatile::Protect,
            "endure" => Volatile::Endure,
            "substitute" => Volatile::Substitute,

            "fly" | "bounce" | "skydrop" => Volatile::Fly,
            "dig" => Volatile::Dig,
            "dive" => Volatile::Dive,
            "shadowforce" => Volatile::ShadowForce,
            "phantomforce" => Volatile::PhantomForce,

            "flinch" => Volatile::Flinch,
            "yawn" => Volatile::Yawn,
            "mustrecharge" | "recharging" => Volatile::Recharging,
            "twoturnmove" | "charging" | "solarbeam" | "razorwind" | "skullbash" | "skyattack"
            | "freezeshock" | "iceburn" | "geomancy" | "meteorbeam" | "electroshot" => {
                Volatile::Charging
            }

            "bide" => Volatile::Bide,
            "uproar" => Volatile::Uproar,
            "lockedmove" | "thrash" | "outrage" | "petaldance" => Volatile::Thrash,
            "rollout" | "iceball" => Volatile::Rollout,

            "magnetrise" => Volatile::MagnetRise,
            "telekinesis" => Volatile::Telekinesis,
            "smackdown" => Volatile::Smackdown,
            "ingrain" => Volatile::Ingrain,
            "aquaring" => Volatile::AquaRing,

            "flashfire" => Volatile::FlashFire,
            "slowstart" => Volatile::SlowStart,
            "truant" => Volatile::Truant,
            "unburden" => Volatile::Unburden,
            "gastroacid" => Volatile::GastroAcid,
            "imprison" => Volatile::Imprison,
            "minimize" => Volatile::Minimize,
            "defensecurl" => Volatile::DefenseCurl,

            "transform" | "transformed" => Volatile::Transformed,

            "roost" => Volatile::Roost,
            "stockpile" | "stockpile1" | "stockpile2" | "stockpile3" => Volatile::Stockpile,
            "helpinghand" => Volatile::HelpingHand,
            "powertrick" => Volatile::PowerTrick,
            "autotomize" => Volatile::Autotomize,
            "magiccoat" => Volatile::MagicCoat,
            "snatch" => Volatile::Snatch,
            "destinybond" => Volatile::DestinyBond,
            "grudge" => Volatile::Grudge,
            "rage" => Volatile::Rage,
            "focuspunch" => Volatile::FocusPunch,
            "mudsport" => Volatile::MudSport,
            "watersport" => Volatile::WaterSport,
            "electrify" => Volatile::Electrify,
            "followme" | "ragepowder" | "centerofattention" | "spotlight" => {
                Volatile::CenterOfAttention
            }

            "dynamax" | "dynamaxed" => Volatile::Dynamaxed,
            "octolock" => Volatile::Octolock,
            "tarshot" => Volatile::TarShot,
            "noretreat" => Volatile::NoRetreat,

            "terastallized" | "tera" => Volatile::Terastallized,
            "saltcure" => Volatile::SaltCure,
            "syrupy" | "syrupbomb" => Volatile::Syrupy,

            // Unknown volatile
            _ => Volatile::Other(s.to_string()),
        }
    }

    /// Check if this is a known volatile (not Other)
    pub fn is_known(&self) -> bool {
        !matches!(self, Volatile::Other(_))
    }

    /// Get display name
    pub fn as_str(&self) -> &str {
        match self {
            Volatile::Trapped => "Trapped",
            Volatile::PartialTrap => "Partial Trap",
            Volatile::Confusion => "Confusion",
            Volatile::Taunt => "Taunt",
            Volatile::Encore => "Encore",
            Volatile::Disable => "Disable",
            Volatile::Torment => "Torment",
            Volatile::Infatuation => "Infatuation",
            Volatile::FocusEnergy => "Focus Energy",
            Volatile::LaserFocus => "Laser Focus",
            Volatile::LeechSeed => "Leech Seed",
            Volatile::Curse => "Curse",
            Volatile::PerishSong => "Perish Song",
            Volatile::Nightmare => "Nightmare",
            Volatile::Protect => "Protect",
            Volatile::Endure => "Endure",
            Volatile::Substitute => "Substitute",
            Volatile::Fly => "Fly",
            Volatile::Dig => "Dig",
            Volatile::Dive => "Dive",
            Volatile::ShadowForce => "Shadow Force",
            Volatile::PhantomForce => "Phantom Force",
            Volatile::Bounce => "Bounce",
            Volatile::SkyDrop => "Sky Drop",
            Volatile::Flinch => "Flinch",
            Volatile::Yawn => "Yawn",
            Volatile::Recharging => "Recharging",
            Volatile::Charging => "Charging",
            Volatile::Bide => "Bide",
            Volatile::Uproar => "Uproar",
            Volatile::Thrash => "Thrash",
            Volatile::Rollout => "Rollout",
            Volatile::MagnetRise => "Magnet Rise",
            Volatile::Telekinesis => "Telekinesis",
            Volatile::Smackdown => "Smack Down",
            Volatile::Ingrain => "Ingrain",
            Volatile::AquaRing => "Aqua Ring",
            Volatile::FlashFire => "Flash Fire",
            Volatile::SlowStart => "Slow Start",
            Volatile::Truant => "Truant",
            Volatile::Unburden => "Unburden",
            Volatile::GastroAcid => "Gastro Acid",
            Volatile::Imprison => "Imprison",
            Volatile::Minimize => "Minimize",
            Volatile::DefenseCurl => "Defense Curl",
            Volatile::Transformed => "Transformed",
            Volatile::Roost => "Roost",
            Volatile::Stockpile => "Stockpile",
            Volatile::HelpingHand => "Helping Hand",
            Volatile::PowerTrick => "Power Trick",
            Volatile::Autotomize => "Autotomize",
            Volatile::MagicCoat => "Magic Coat",
            Volatile::Snatch => "Snatch",
            Volatile::DestinyBond => "Destiny Bond",
            Volatile::Grudge => "Grudge",
            Volatile::Rage => "Rage",
            Volatile::FocusPunch => "Focus Punch",
            Volatile::MudSport => "Mud Sport",
            Volatile::WaterSport => "Water Sport",
            Volatile::Electrify => "Electrify",
            Volatile::CenterOfAttention => "Center of Attention",
            Volatile::Dynamaxed => "Dynamaxed",
            Volatile::Octolock => "Octolock",
            Volatile::TarShot => "Tar Shot",
            Volatile::NoRetreat => "No Retreat",
            Volatile::Terastallized => "Terastallized",
            Volatile::SaltCure => "Salt Cure",
            Volatile::Syrupy => "Syrupy",
            Volatile::Other(s) => s.as_str(),
        }
    }
}

impl std::fmt::Display for Volatile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_from_protocol() {
        assert_eq!(Status::from_protocol("brn"), Some(Status::Burn));
        assert_eq!(Status::from_protocol("frz"), Some(Status::Freeze));
        assert_eq!(Status::from_protocol("par"), Some(Status::Paralysis));
        assert_eq!(Status::from_protocol("psn"), Some(Status::Poison));
        assert_eq!(Status::from_protocol("tox"), Some(Status::BadPoison));
        assert_eq!(Status::from_protocol("slp"), Some(Status::Sleep));
        assert_eq!(Status::from_protocol("fnt"), None);
        assert_eq!(Status::from_protocol("unknown"), None);
    }

    #[test]
    fn test_status_to_protocol() {
        assert_eq!(Status::Burn.to_protocol(), "brn");
        assert_eq!(Status::BadPoison.to_protocol(), "tox");
    }

    #[test]
    fn test_volatile_from_protocol_basic() {
        assert_eq!(Volatile::from_protocol("confusion"), Volatile::Confusion);
        assert_eq!(Volatile::from_protocol("Confusion"), Volatile::Confusion);
        assert_eq!(Volatile::from_protocol("taunt"), Volatile::Taunt);
        assert_eq!(Volatile::from_protocol("substitute"), Volatile::Substitute);
    }

    #[test]
    fn test_volatile_from_protocol_with_prefix() {
        assert_eq!(Volatile::from_protocol("move: Taunt"), Volatile::Taunt);
        assert_eq!(
            Volatile::from_protocol("ability: Flash Fire"),
            Volatile::FlashFire
        );
    }

    #[test]
    fn test_volatile_from_protocol_unknown() {
        let v = Volatile::from_protocol("some_unknown_volatile");
        assert_eq!(v, Volatile::Other("some_unknown_volatile".to_string()));
        assert!(!v.is_known());
    }

    #[test]
    fn test_volatile_is_known() {
        assert!(Volatile::Confusion.is_known());
        assert!(Volatile::Substitute.is_known());
        assert!(!Volatile::Other("test".to_string()).is_known());
    }

    #[test]
    fn test_volatile_protect_variants() {
        assert_eq!(Volatile::from_protocol("protect"), Volatile::Protect);
        assert_eq!(Volatile::from_protocol("detect"), Volatile::Protect);
        assert_eq!(Volatile::from_protocol("King's Shield"), Volatile::Protect);
        assert_eq!(Volatile::from_protocol("spikyshield"), Volatile::Protect);
    }
}
