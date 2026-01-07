//! Battle request types
//!
//! These types represent the JSON structure of |request| messages.

use super::battle::Player;
use serde::Deserialize;

/// A battle request asking the player to make a decision
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BattleRequest {
    /// Request ID for synchronization
    pub rqid: Option<u64>,

    /// Active pokemon and their available moves
    #[serde(default)]
    pub active: Option<Vec<ActivePokemon>>,

    /// Information about the player's side/team
    pub side: Option<SideInfo>,

    /// Which slots need to switch (for doubles/triples)
    #[serde(default)]
    pub force_switch: Option<Vec<bool>>,

    /// Whether this is team preview
    #[serde(default)]
    pub team_preview: bool,

    /// Whether we're waiting for opponent
    #[serde(default)]
    pub wait: bool,

    /// No action needed (e.g., between turns)
    #[serde(default)]
    pub no_cancel: bool,
}

impl BattleRequest {
    /// Parse a request from JSON
    pub fn parse(json: &serde_json::Value) -> Option<Self> {
        serde_json::from_value(json.clone()).ok()
    }

    /// Check if this request requires a decision
    pub fn needs_decision(&self) -> bool {
        !self.wait && (self.team_preview || self.force_switch.is_some() || self.active.is_some())
    }

    /// Check if this is a force switch request
    pub fn is_force_switch(&self) -> bool {
        self.force_switch
            .as_ref()
            .map(|fs| fs.iter().any(|&b| b))
            .unwrap_or(false)
    }

    /// Get available pokemon to switch to
    pub fn available_switches(&self) -> Vec<&SidePokemon> {
        self.side
            .as_ref()
            .map(|s| {
                s.pokemon
                    .iter()
                    .filter(|p| !p.active && !p.is_fainted())
                    .collect()
            })
            .unwrap_or_default()
    }
}

/// Information about an active pokemon in battle
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivePokemon {
    /// Available moves
    #[serde(default)]
    pub moves: Vec<MoveSlot>,

    /// Whether the pokemon is trapped
    #[serde(default)]
    pub trapped: bool,

    /// Whether the pokemon might be trapped
    #[serde(default)]
    pub maybe_trapped: bool,

    /// Whether mega evolution is available
    #[serde(default)]
    pub can_mega_evo: bool,

    /// Whether ultra burst is available
    #[serde(default)]
    pub can_ultra_burst: bool,

    /// Z-move information (if available)
    #[serde(default)]
    pub can_z_move: Option<Vec<Option<ZMoveInfo>>>,

    /// Whether dynamax is available
    #[serde(default)]
    pub can_dynamax: bool,

    /// Whether gigantamax is available
    #[serde(default)]
    pub can_gigantamax: Option<String>,

    /// Terastallization type (if available)
    #[serde(default)]
    pub can_terastallize: Option<String>,

    /// Max moves (when dynamaxed)
    #[serde(default)]
    pub max_moves: Option<MaxMoves>,
}

impl ActivePokemon {
    /// Get available (non-disabled, with PP) moves
    pub fn available_moves(&self) -> Vec<(usize, &MoveSlot)> {
        self.moves
            .iter()
            .enumerate()
            .filter(|(_, m)| !m.disabled && m.pp > 0)
            .collect()
    }

    /// Check if the pokemon can switch out
    pub fn can_switch(&self) -> bool {
        !self.trapped && !self.maybe_trapped
    }
}

/// A move slot on an active pokemon
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoveSlot {
    /// Display name of the move
    #[serde(rename = "move")]
    pub name: String,

    /// Move ID (lowercase, no spaces)
    pub id: String,

    /// Current PP
    pub pp: u32,

    /// Maximum PP
    #[serde(rename = "maxpp")]
    pub max_pp: u32,

    /// Target type (normal, self, allySide, etc.)
    #[serde(default)]
    pub target: String,

    /// Whether the move is disabled
    #[serde(default)]
    pub disabled: bool,
}

/// Z-move information
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ZMoveInfo {
    /// Z-move name
    #[serde(rename = "move")]
    pub name: String,

    /// Target type
    pub target: String,
}

/// Max move information (for dynamax)
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MaxMoves {
    /// Available max moves
    #[serde(default)]
    pub max_moves: Vec<MaxMoveSlot>,
}

/// A max move slot
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MaxMoveSlot {
    /// Max move name
    #[serde(rename = "move")]
    pub name: String,

    /// Target type
    pub target: String,
}

/// Information about the player's side
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SideInfo {
    /// Player's display name
    pub name: String,

    /// Player ID (p1, p2, etc.)
    pub id: String,

    /// Pokemon on this side
    #[serde(default)]
    pub pokemon: Vec<SidePokemon>,
}

impl SideInfo {
    /// Get the player enum
    pub fn player(&self) -> Option<Player> {
        Player::parse(&self.id)
    }
}

/// A pokemon on the player's side
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SidePokemon {
    /// Pokemon identifier (e.g., "p1: Pikachu")
    pub ident: String,

    /// Details string (species, level, gender, shiny)
    pub details: String,

    /// Current condition (HP/MaxHP status)
    pub condition: String,

    /// Whether this pokemon is currently active
    #[serde(default)]
    pub active: bool,

    /// Stats (atk, def, spa, spd, spe)
    #[serde(default)]
    pub stats: PokemonStats,

    /// Known moves
    #[serde(default)]
    pub moves: Vec<String>,

    /// Base ability
    #[serde(default)]
    pub base_ability: String,

    /// Current ability
    #[serde(default)]
    pub ability: String,

    /// Held item
    #[serde(default)]
    pub item: String,

    /// Pokeball used
    #[serde(default)]
    pub pokeball: String,

    /// Terastallize type
    #[serde(default)]
    pub teratype: Option<String>,

    /// Whether already terastallized
    #[serde(default)]
    pub terastallized: Option<String>,
}

impl SidePokemon {
    /// Check if the pokemon is fainted
    pub fn is_fainted(&self) -> bool {
        self.condition == "0 fnt" || self.condition.ends_with(" fnt")
    }

    /// Get current HP as a fraction (current, max)
    pub fn hp(&self) -> Option<(u32, u32)> {
        let hp_part = self.condition.split_whitespace().next()?;
        let (current, max) = hp_part.split_once('/')?;
        Some((current.parse().ok()?, max.parse().ok()?))
    }

    /// Get HP as a percentage (0-100)
    pub fn hp_percent(&self) -> u32 {
        self.hp()
            .map(|(cur, max)| if max > 0 { cur * 100 / max } else { 0 })
            .unwrap_or(0)
    }

    /// Get the status condition (if any)
    pub fn status(&self) -> Option<&str> {
        self.condition.split_whitespace().nth(1)
    }

    /// Get the species name from details
    pub fn species(&self) -> &str {
        self.details.split(',').next().unwrap_or(&self.details)
    }
}

/// Pokemon stats
#[derive(Debug, Clone, PartialEq, Default, Deserialize)]
pub struct PokemonStats {
    pub atk: u32,
    pub def: u32,
    pub spa: u32,
    pub spd: u32,
    pub spe: u32,
}
