//! TrackedBattle - main battle state tracking struct

use kazam_protocol::{GameType, Player};

use crate::types::{FieldState, SideState};

/// A battle being tracked from server messages
///
/// This struct reconstructs battle state from the protocol messages
/// received from the Pokemon Showdown server. It maintains the perspective
/// of one player and tracks what information has been revealed.
#[derive(Debug, Clone)]
pub struct TrackedBattle {
    // === Battle metadata ===
    /// Game type (singles, doubles, etc.)
    pub game_type: Option<GameType>,

    /// Generation (1-9)
    pub generation: u8,

    /// Format/tier name
    pub tier: String,

    /// Current turn number (0 = not started)
    pub turn: u32,

    // === State ===
    /// Global field state (weather, terrain, etc.)
    pub field: FieldState,

    /// Player sides (indexed by Player enum)
    /// Up to 4 players for multi battles
    pub(crate) sides: [Option<SideState>; 4],

    // === Perspective ===
    /// Which player we are (for me()/opponent() methods)
    perspective: Option<Player>,

    // === Outcome ===
    /// Whether the battle has ended
    pub ended: bool,

    /// Winner's username (if ended)
    pub winner: Option<String>,

    /// Whether the battle ended in a tie
    pub tie: bool,
}

impl TrackedBattle {
    /// Create a new battle tracker
    pub fn new() -> Self {
        Self {
            game_type: None,
            generation: 9, // Default to latest gen
            tier: String::new(),
            turn: 0,
            field: FieldState::new(),
            sides: [None, None, None, None],
            perspective: None,
            ended: false,
            winner: None,
            tie: false,
        }
    }

    /// Set the perspective (which player we are)
    pub fn set_perspective(&mut self, player: Player) {
        self.perspective = Some(player);
    }

    /// Get the current perspective
    pub fn perspective(&self) -> Option<Player> {
        self.perspective
    }

    /// Get our side (based on perspective)
    pub fn me(&self) -> Option<&SideState> {
        self.perspective.and_then(|p| self.get_side(p))
    }

    /// Get our side mutably
    pub fn me_mut(&mut self) -> Option<&mut SideState> {
        self.perspective.and_then(|p| self.get_side_mut(p))
    }

    /// Get opponent's side (assumes 1v1 battle)
    pub fn opponent(&self) -> Option<&SideState> {
        let opp = self.opponent_player()?;
        self.get_side(opp)
    }

    /// Get opponent's side mutably
    pub fn opponent_mut(&mut self) -> Option<&mut SideState> {
        let opp = self.opponent_player()?;
        self.get_side_mut(opp)
    }

    /// Get the opponent player (assumes 1v1)
    fn opponent_player(&self) -> Option<Player> {
        match self.perspective? {
            Player::P1 => Some(Player::P2),
            Player::P2 => Some(Player::P1),
            Player::P3 => Some(Player::P4),
            Player::P4 => Some(Player::P3),
        }
    }

    /// Get a side by player
    pub fn get_side(&self, player: Player) -> Option<&SideState> {
        let idx = player_to_index(player);
        self.sides[idx].as_ref()
    }

    /// Get a side mutably by player
    pub fn get_side_mut(&mut self, player: Player) -> Option<&mut SideState> {
        let idx = player_to_index(player);
        self.sides[idx].as_mut()
    }

    /// Get or create a side for a player
    pub fn get_or_create_side(&mut self, player: Player, username: &str) -> &mut SideState {
        let idx = player_to_index(player);
        if self.sides[idx].is_none() {
            self.sides[idx] = Some(SideState::new(player, username));
        }
        self.sides[idx].as_mut().unwrap()
    }

    /// Check if a side exists
    pub fn has_side(&self, player: Player) -> bool {
        let idx = player_to_index(player);
        self.sides[idx].is_some()
    }

    /// Iterate over all initialized sides
    pub fn sides(&self) -> impl Iterator<Item = &SideState> {
        self.sides.iter().filter_map(|s| s.as_ref())
    }

    /// Iterate over all initialized sides mutably
    pub fn sides_mut(&mut self) -> impl Iterator<Item = &mut SideState> {
        self.sides.iter_mut().filter_map(|s| s.as_mut())
    }

    /// Set game type and update active slots accordingly
    pub fn set_game_type(&mut self, game_type: GameType) {
        self.game_type = Some(game_type);

        let slots = match game_type {
            GameType::Singles => 1,
            GameType::Doubles => 2,
            GameType::Triples => 3,
            GameType::Multi => 2,
            GameType::FreeForAll => 1,
        };

        for side in self.sides_mut() {
            side.set_active_slots(slots);
        }
    }

    /// Check if the battle is in progress
    pub fn is_active(&self) -> bool {
        self.turn > 0 && !self.ended
    }

    /// Check if we're waiting for the battle to start
    pub fn is_waiting_to_start(&self) -> bool {
        self.turn == 0 && !self.ended
    }

    /// Get all active Pokemon from all sides in speed order (not implemented yet)
    pub fn get_all_active(&self) -> Vec<&crate::types::PokemonState> {
        self.sides()
            .flat_map(|side| side.get_active())
            .collect()
    }
}

impl Default for TrackedBattle {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert Player enum to array index
pub fn player_to_index(player: Player) -> usize {
    match player {
        Player::P1 => 0,
        Player::P2 => 1,
        Player::P3 => 2,
        Player::P4 => 3,
    }
}

/// Convert position character to slot index
pub fn position_to_slot(pos: char) -> usize {
    match pos {
        'a' => 0,
        'b' => 1,
        'c' => 2,
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_battle() {
        let battle = TrackedBattle::new();
        assert_eq!(battle.turn, 0);
        assert!(!battle.ended);
        assert!(battle.perspective.is_none());
        assert!(battle.game_type.is_none());
    }

    #[test]
    fn test_set_perspective() {
        let mut battle = TrackedBattle::new();
        battle.set_perspective(Player::P1);
        assert_eq!(battle.perspective(), Some(Player::P1));
    }

    #[test]
    fn test_get_or_create_side() {
        let mut battle = TrackedBattle::new();

        assert!(!battle.has_side(Player::P1));

        let side = battle.get_or_create_side(Player::P1, "Alice");
        assert_eq!(side.username, "Alice");

        assert!(battle.has_side(Player::P1));
    }

    #[test]
    fn test_me_and_opponent() {
        let mut battle = TrackedBattle::new();

        // Set up sides
        battle.get_or_create_side(Player::P1, "Alice");
        battle.get_or_create_side(Player::P2, "Bob");

        // Before perspective is set
        assert!(battle.me().is_none());
        assert!(battle.opponent().is_none());

        // Set perspective
        battle.set_perspective(Player::P1);

        let me = battle.me().unwrap();
        assert_eq!(me.username, "Alice");

        let opp = battle.opponent().unwrap();
        assert_eq!(opp.username, "Bob");
    }

    #[test]
    fn test_set_game_type() {
        let mut battle = TrackedBattle::new();
        battle.get_or_create_side(Player::P1, "Test");

        battle.set_game_type(GameType::Singles);
        assert_eq!(
            battle.get_side(Player::P1).unwrap().active_indices.len(),
            1
        );

        battle.set_game_type(GameType::Doubles);
        assert_eq!(
            battle.get_side(Player::P1).unwrap().active_indices.len(),
            2
        );
    }

    #[test]
    fn test_is_active() {
        let mut battle = TrackedBattle::new();

        assert!(!battle.is_active());
        assert!(battle.is_waiting_to_start());

        battle.turn = 1;
        assert!(battle.is_active());
        assert!(!battle.is_waiting_to_start());

        battle.ended = true;
        assert!(!battle.is_active());
    }

    #[test]
    fn test_player_to_index() {
        assert_eq!(player_to_index(Player::P1), 0);
        assert_eq!(player_to_index(Player::P2), 1);
        assert_eq!(player_to_index(Player::P3), 2);
        assert_eq!(player_to_index(Player::P4), 3);
    }

    #[test]
    fn test_position_to_slot() {
        assert_eq!(position_to_slot('a'), 0);
        assert_eq!(position_to_slot('b'), 1);
        assert_eq!(position_to_slot('c'), 2);
        assert_eq!(position_to_slot('d'), 0); // Default
    }
}
