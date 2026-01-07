//! Battle state types
//!
//! These types track the state of a battle room.

use super::battle::{GameType, Player};

/// Information about a battle, collected during initialization
#[derive(Debug, Clone, PartialEq, Default)]
pub struct BattleInfo {
    /// Players in the battle
    pub players: Vec<PlayerInfo>,

    /// Type of battle (singles, doubles, etc.)
    pub game_type: Option<GameType>,

    /// Generation number
    pub generation: u8,

    /// Format/tier name
    pub tier: String,

    /// Whether the battle is rated
    pub rated: bool,

    /// Custom rated message (for tournaments, etc.)
    pub rated_message: Option<String>,

    /// Active rules
    pub rules: Vec<String>,

    /// Team preview pokemon (before battle starts)
    pub preview: Vec<PreviewPokemon>,

    /// Whether the battle has started
    pub started: bool,

    /// Current turn number
    pub turn: u32,

    /// Battle winner (if ended)
    pub winner: Option<String>,

    /// Whether battle ended in tie
    pub tie: bool,
}

impl BattleInfo {
    /// Create a new empty battle info
    pub fn new() -> Self {
        Self::default()
    }

    /// Get player info by player ID
    pub fn get_player(&self, player: Player) -> Option<&PlayerInfo> {
        self.players.iter().find(|p| p.player == player)
    }

    /// Check if the battle has ended
    pub fn is_ended(&self) -> bool {
        self.winner.is_some() || self.tie
    }
}

/// Information about a player in a battle
#[derive(Debug, Clone, PartialEq)]
pub struct PlayerInfo {
    /// Player ID (p1, p2, etc.)
    pub player: Player,

    /// Player's username
    pub username: String,

    /// Player's avatar
    pub avatar: String,

    /// Player's rating (if rated)
    pub rating: Option<u32>,

    /// Team size
    pub team_size: u8,
}

/// Pokemon shown in team preview
#[derive(Debug, Clone, PartialEq)]
pub struct PreviewPokemon {
    /// Which player owns this pokemon
    pub player: Player,

    /// Species name (may be hidden as Species-*)
    pub species: String,

    /// Level (if known)
    pub level: Option<u8>,

    /// Gender (if known)
    pub gender: Option<char>,

    /// Whether holding an item
    pub has_item: bool,
}
