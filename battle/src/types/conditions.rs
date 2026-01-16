//! Field and side conditions

/// Weather conditions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Weather {
    Sun,
    Rain,
    Sand,
    Hail,
    Snow,        // Gen 9 replacement for Hail
    HarshSun,    // Desolate Land (Primal Groudon)
    HeavyRain,   // Primordial Sea (Primal Kyogre)
    StrongWinds, // Delta Stream (Mega Rayquaza)
}

impl Weather {
    /// Parse from protocol string
    pub fn from_protocol(s: &str) -> Option<Self> {
        // Normalize: lowercase and remove spaces
        let normalized = s.to_lowercase().replace([' ', '-'], "");

        match normalized.as_str() {
            "sunnyday" | "sun" | "harshsunlight" => Some(Weather::Sun),
            "raindance" | "rain" => Some(Weather::Rain),
            "sandstorm" | "sand" => Some(Weather::Sand),
            "hail" => Some(Weather::Hail),
            "snow" => Some(Weather::Snow),
            "desolateland" | "harshsun" | "extremelyharshsunlight" => Some(Weather::HarshSun),
            "primordialsea" | "heavyrain" => Some(Weather::HeavyRain),
            "deltastream" | "strongwinds" | "mysteriousaircurrent" => Some(Weather::StrongWinds),
            "none" | "" => None,
            _ => None,
        }
    }

    /// Check if this is a primal weather (cannot be overwritten by normal weather)
    pub fn is_primal(&self) -> bool {
        matches!(
            self,
            Weather::HarshSun | Weather::HeavyRain | Weather::StrongWinds
        )
    }

    /// Get display name
    pub fn as_str(&self) -> &'static str {
        match self {
            Weather::Sun => "Sun",
            Weather::Rain => "Rain",
            Weather::Sand => "Sandstorm",
            Weather::Hail => "Hail",
            Weather::Snow => "Snow",
            Weather::HarshSun => "Harsh Sun",
            Weather::HeavyRain => "Heavy Rain",
            Weather::StrongWinds => "Strong Winds",
        }
    }
}

impl std::fmt::Display for Weather {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Terrain conditions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Terrain {
    Electric,
    Grassy,
    Misty,
    Psychic,
}

impl Terrain {
    /// Parse from protocol string
    pub fn from_protocol(s: &str) -> Option<Self> {
        // Strip common prefixes
        let clean = s
            .strip_prefix("move: ")
            .unwrap_or(s);

        // Normalize
        let normalized = clean.to_lowercase().replace([' ', '-'], "");

        match normalized.as_str() {
            "electricterrain" | "electric" => Some(Terrain::Electric),
            "grassyterrain" | "grassy" => Some(Terrain::Grassy),
            "mistyterrain" | "misty" => Some(Terrain::Misty),
            "psychicterrain" | "psychic" => Some(Terrain::Psychic),
            "none" | "" => None,
            _ => None,
        }
    }

    /// Get display name
    pub fn as_str(&self) -> &'static str {
        match self {
            Terrain::Electric => "Electric Terrain",
            Terrain::Grassy => "Grassy Terrain",
            Terrain::Misty => "Misty Terrain",
            Terrain::Psychic => "Psychic Terrain",
        }
    }
}

impl std::fmt::Display for Terrain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Side conditions (hazards, screens, etc.)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SideCondition {
    // Screens
    Reflect,
    LightScreen,
    AuroraVeil,

    // Entry hazards
    Spikes,      // Stackable 1-3
    ToxicSpikes, // Stackable 1-2
    StealthRock,
    StickyWeb,

    // Other
    Tailwind,
    Safeguard,
    Mist,
    LuckyChant,
    WideGuard,
    QuickGuard,
    MatBlock,
}

impl SideCondition {
    /// Parse from protocol string
    pub fn from_protocol(s: &str) -> Option<Self> {
        // Strip common prefixes
        let clean = s
            .strip_prefix("move: ")
            .unwrap_or(s);

        // Normalize
        let normalized = clean.to_lowercase().replace([' ', '-'], "");

        match normalized.as_str() {
            "reflect" => Some(SideCondition::Reflect),
            "lightscreen" => Some(SideCondition::LightScreen),
            "auroraveil" => Some(SideCondition::AuroraVeil),
            "spikes" => Some(SideCondition::Spikes),
            "toxicspikes" => Some(SideCondition::ToxicSpikes),
            "stealthrock" => Some(SideCondition::StealthRock),
            "stickyweb" => Some(SideCondition::StickyWeb),
            "tailwind" => Some(SideCondition::Tailwind),
            "safeguard" => Some(SideCondition::Safeguard),
            "mist" => Some(SideCondition::Mist),
            "luckychant" => Some(SideCondition::LuckyChant),
            "wideguard" => Some(SideCondition::WideGuard),
            "quickguard" => Some(SideCondition::QuickGuard),
            "matblock" => Some(SideCondition::MatBlock),
            _ => None,
        }
    }

    /// Check if this condition is stackable
    pub fn is_stackable(&self) -> bool {
        matches!(self, SideCondition::Spikes | SideCondition::ToxicSpikes)
    }

    /// Get maximum layers for this condition
    pub fn max_layers(&self) -> u8 {
        match self {
            SideCondition::Spikes => 3,
            SideCondition::ToxicSpikes => 2,
            _ => 1,
        }
    }

    /// Check if this is a screen
    pub fn is_screen(&self) -> bool {
        matches!(
            self,
            SideCondition::Reflect | SideCondition::LightScreen | SideCondition::AuroraVeil
        )
    }

    /// Check if this is an entry hazard
    pub fn is_hazard(&self) -> bool {
        matches!(
            self,
            SideCondition::Spikes
                | SideCondition::ToxicSpikes
                | SideCondition::StealthRock
                | SideCondition::StickyWeb
        )
    }

    /// Get display name
    pub fn as_str(&self) -> &'static str {
        match self {
            SideCondition::Reflect => "Reflect",
            SideCondition::LightScreen => "Light Screen",
            SideCondition::AuroraVeil => "Aurora Veil",
            SideCondition::Spikes => "Spikes",
            SideCondition::ToxicSpikes => "Toxic Spikes",
            SideCondition::StealthRock => "Stealth Rock",
            SideCondition::StickyWeb => "Sticky Web",
            SideCondition::Tailwind => "Tailwind",
            SideCondition::Safeguard => "Safeguard",
            SideCondition::Mist => "Mist",
            SideCondition::LuckyChant => "Lucky Chant",
            SideCondition::WideGuard => "Wide Guard",
            SideCondition::QuickGuard => "Quick Guard",
            SideCondition::MatBlock => "Mat Block",
        }
    }
}

impl std::fmt::Display for SideCondition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// State for a side condition (tracks layers for stackable conditions)
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SideConditionState {
    pub layers: u8,
}

impl SideConditionState {
    /// Create a new condition state with 1 layer
    pub fn new() -> Self {
        Self { layers: 1 }
    }

    /// Add a layer, returns true if successful
    pub fn add_layer(&mut self, condition: SideCondition) -> bool {
        if self.layers < condition.max_layers() {
            self.layers += 1;
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weather_from_protocol() {
        assert_eq!(Weather::from_protocol("SunnyDay"), Some(Weather::Sun));
        assert_eq!(Weather::from_protocol("RainDance"), Some(Weather::Rain));
        assert_eq!(Weather::from_protocol("Sandstorm"), Some(Weather::Sand));
        assert_eq!(Weather::from_protocol("Hail"), Some(Weather::Hail));
        assert_eq!(Weather::from_protocol("Snow"), Some(Weather::Snow));
        assert_eq!(
            Weather::from_protocol("DesolateLand"),
            Some(Weather::HarshSun)
        );
        assert_eq!(
            Weather::from_protocol("PrimordialSea"),
            Some(Weather::HeavyRain)
        );
        assert_eq!(
            Weather::from_protocol("DeltaStream"),
            Some(Weather::StrongWinds)
        );
        assert_eq!(Weather::from_protocol("none"), None);
    }

    #[test]
    fn test_weather_is_primal() {
        assert!(!Weather::Sun.is_primal());
        assert!(!Weather::Rain.is_primal());
        assert!(Weather::HarshSun.is_primal());
        assert!(Weather::HeavyRain.is_primal());
        assert!(Weather::StrongWinds.is_primal());
    }

    #[test]
    fn test_terrain_from_protocol() {
        assert_eq!(
            Terrain::from_protocol("Electric Terrain"),
            Some(Terrain::Electric)
        );
        assert_eq!(
            Terrain::from_protocol("move: Grassy Terrain"),
            Some(Terrain::Grassy)
        );
        assert_eq!(
            Terrain::from_protocol("MistyTerrain"),
            Some(Terrain::Misty)
        );
        assert_eq!(
            Terrain::from_protocol("psychicterrain"),
            Some(Terrain::Psychic)
        );
        assert_eq!(Terrain::from_protocol("none"), None);
    }

    #[test]
    fn test_side_condition_from_protocol() {
        assert_eq!(
            SideCondition::from_protocol("Stealth Rock"),
            Some(SideCondition::StealthRock)
        );
        assert_eq!(
            SideCondition::from_protocol("move: Reflect"),
            Some(SideCondition::Reflect)
        );
        assert_eq!(
            SideCondition::from_protocol("Spikes"),
            Some(SideCondition::Spikes)
        );
        assert_eq!(
            SideCondition::from_protocol("toxicspikes"),
            Some(SideCondition::ToxicSpikes)
        );
    }

    #[test]
    fn test_side_condition_stackable() {
        assert!(SideCondition::Spikes.is_stackable());
        assert!(SideCondition::ToxicSpikes.is_stackable());
        assert!(!SideCondition::StealthRock.is_stackable());
        assert!(!SideCondition::Reflect.is_stackable());
    }

    #[test]
    fn test_side_condition_max_layers() {
        assert_eq!(SideCondition::Spikes.max_layers(), 3);
        assert_eq!(SideCondition::ToxicSpikes.max_layers(), 2);
        assert_eq!(SideCondition::StealthRock.max_layers(), 1);
    }

    #[test]
    fn test_side_condition_is_screen() {
        assert!(SideCondition::Reflect.is_screen());
        assert!(SideCondition::LightScreen.is_screen());
        assert!(SideCondition::AuroraVeil.is_screen());
        assert!(!SideCondition::Spikes.is_screen());
    }

    #[test]
    fn test_side_condition_is_hazard() {
        assert!(SideCondition::Spikes.is_hazard());
        assert!(SideCondition::ToxicSpikes.is_hazard());
        assert!(SideCondition::StealthRock.is_hazard());
        assert!(SideCondition::StickyWeb.is_hazard());
        assert!(!SideCondition::Reflect.is_hazard());
    }

    #[test]
    fn test_side_condition_state() {
        let mut state = SideConditionState::new();
        assert_eq!(state.layers, 1);

        // Can add layers to Spikes (max 3)
        assert!(state.add_layer(SideCondition::Spikes));
        assert_eq!(state.layers, 2);
        assert!(state.add_layer(SideCondition::Spikes));
        assert_eq!(state.layers, 3);
        assert!(!state.add_layer(SideCondition::Spikes)); // At max
        assert_eq!(state.layers, 3);
    }
}
