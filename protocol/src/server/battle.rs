//! Shared types for battle protocol messages

use crate::ParseError;

/// Player in a battle (p1, p2, p3, p4)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Player {
    P1,
    P2,
    P3,
    P4,
}

impl Player {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "p1" => Some(Player::P1),
            "p2" => Some(Player::P2),
            "p3" => Some(Player::P3),
            "p4" => Some(Player::P4),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Player::P1 => "p1",
            Player::P2 => "p2",
            Player::P3 => "p3",
            Player::P4 => "p4",
        }
    }
}

/// Pokemon identifier in the form "POSITION: NAME" (e.g., "p1a: Pikachu")
#[derive(Debug, Clone, PartialEq)]
pub struct Pokemon {
    /// Player who owns this pokemon
    pub player: Player,
    /// Position letter (a, b, c for active slots, or None if inactive)
    pub position: Option<char>,
    /// Pokemon's name/nickname
    pub name: String,
}

impl Pokemon {
    /// Parse a pokemon ID string like "p1a: Pikachu" or "p1: Pikachu"
    pub fn parse(s: &str) -> Option<Self> {
        let (pos_part, name) = s.split_once(": ")?;

        let player = if pos_part.starts_with("p1") {
            Player::P1
        } else if pos_part.starts_with("p2") {
            Player::P2
        } else if pos_part.starts_with("p3") {
            Player::P3
        } else if pos_part.starts_with("p4") {
            Player::P4
        } else {
            return None;
        };

        let position = pos_part.chars().nth(2);

        Some(Pokemon {
            player,
            position,
            name: name.to_string(),
        })
    }
}

/// Pokemon details string (species, level, gender, shiny, tera)
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PokemonDetails {
    pub species: String,
    pub level: Option<u8>,
    pub gender: Option<char>,
    pub shiny: bool,
    pub tera_type: Option<String>,
}

impl PokemonDetails {
    /// Parse a details string like "Pikachu, L50, M, shiny" or "Arceus-*"
    pub fn parse(s: &str) -> Self {
        let mut details = PokemonDetails::default();
        let parts: Vec<&str> = s.split(", ").collect();

        if let Some(species) = parts.first() {
            details.species = species.to_string();
        }

        for part in parts.iter().skip(1) {
            if let Some(level_str) = part.strip_prefix('L') {
                details.level = level_str.parse().ok();
            } else if *part == "M" {
                details.gender = Some('M');
            } else if *part == "F" {
                details.gender = Some('F');
            } else if *part == "shiny" {
                details.shiny = true;
            } else if let Some(tera) = part.strip_prefix("tera:") {
                details.tera_type = Some(tera.to_string());
            }
        }

        details
    }
}

/// HP and status condition (e.g., "100/100", "50/100 slp", "0 fnt")
#[derive(Debug, Clone, PartialEq)]
pub struct HpStatus {
    /// Current HP (as raw value or percentage depending on context)
    pub current: u32,
    /// Max HP (if known)
    pub max: Option<u32>,
    /// Status condition (slp, par, brn, psn, tox, frz, fnt)
    pub status: Option<String>,
}

impl HpStatus {
    /// Parse an HP status string like "100/100", "50/100 slp", or "0 fnt"
    pub fn parse(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.split_whitespace().collect();
        if parts.is_empty() {
            return None;
        }

        let hp_part = parts[0];
        let status = parts.get(1).map(|s| s.to_string());

        if let Some((current_str, max_str)) = hp_part.split_once('/') {
            Some(HpStatus {
                current: current_str.parse().ok()?,
                max: Some(max_str.parse().ok()?),
                status,
            })
        } else {
            Some(HpStatus {
                current: hp_part.parse().ok()?,
                max: None,
                status,
            })
        }
    }
}

/// Game type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameType {
    Singles,
    Doubles,
    Triples,
    Multi,
    FreeForAll,
}

impl GameType {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "singles" => Some(GameType::Singles),
            "doubles" => Some(GameType::Doubles),
            "triples" => Some(GameType::Triples),
            "multi" => Some(GameType::Multi),
            "freeforall" => Some(GameType::FreeForAll),
            _ => None,
        }
    }
}

/// Stat abbreviation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stat {
    Atk,
    Def,
    Spa,
    Spd,
    Spe,
    Accuracy,
    Evasion,
}

impl Stat {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "atk" => Some(Stat::Atk),
            "def" => Some(Stat::Def),
            "spa" => Some(Stat::Spa),
            "spd" => Some(Stat::Spd),
            "spe" => Some(Stat::Spe),
            "accuracy" => Some(Stat::Accuracy),
            "evasion" => Some(Stat::Evasion),
            _ => None,
        }
    }
}

/// Side of the field (for side conditions)
#[derive(Debug, Clone, PartialEq)]
pub struct Side {
    pub player: Player,
    pub raw: String,
}

impl Side {
    pub fn parse(s: &str) -> Option<Self> {
        let player = if s.starts_with("p1") {
            Player::P1
        } else if s.starts_with("p2") {
            Player::P2
        } else if s.starts_with("p3") {
            Player::P3
        } else if s.starts_with("p4") {
            Player::P4
        } else {
            return None;
        };

        Some(Side {
            player,
            raw: s.to_string(),
        })
    }
}

/// Helper to parse Pokemon from message parts
pub fn parse_pokemon(parts: &[&str], index: usize) -> Result<Pokemon, anyhow::Error> {
    parts
        .get(index)
        .and_then(|s| Pokemon::parse(s))
        .ok_or_else(|| ParseError::MissingField("pokemon".to_string()).into())
}

/// Helper to parse PokemonDetails from message parts
pub fn parse_details(parts: &[&str], index: usize) -> PokemonDetails {
    parts
        .get(index)
        .map(|s| PokemonDetails::parse(s))
        .unwrap_or_default()
}

/// Helper to parse HpStatus from message parts
pub fn parse_hp_status(parts: &[&str], index: usize) -> Option<HpStatus> {
    parts.get(index).and_then(|s| HpStatus::parse(s))
}
