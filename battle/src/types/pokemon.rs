//! Pokemon state types

use std::collections::HashSet;

use kazam_protocol::{HpStatus, PokemonDetails};

use super::pokemon_type::Type;
use super::stats::StatStages;
use super::status::{Status, Volatile};

/// Core Pokemon identity (doesn't change during battle)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PokemonIdentity {
    /// Species name (including forme, e.g., "Pikachu-Alola")
    pub species: String,

    /// Nickname (if different from species)
    pub nickname: Option<String>,

    /// Level (1-100)
    pub level: u8,

    /// Gender ('M', 'F', or None for genderless)
    pub gender: Option<char>,

    /// Whether the Pokemon is shiny
    pub shiny: bool,
}

impl PokemonIdentity {
    /// Create a new Pokemon identity
    pub fn new(species: impl Into<String>, level: u8) -> Self {
        Self {
            species: species.into(),
            nickname: None,
            level,
            gender: None,
            shiny: false,
        }
    }

    /// Create from protocol PokemonDetails
    pub fn from_protocol(details: &PokemonDetails) -> Self {
        Self {
            species: details.species.clone(),
            nickname: None,
            level: details.level.unwrap_or(100),
            gender: details.gender,
            shiny: details.shiny,
        }
    }

    /// Get the display name (nickname if set, otherwise species)
    pub fn name(&self) -> &str {
        self.nickname.as_deref().unwrap_or(&self.species)
    }
}

impl Default for PokemonIdentity {
    fn default() -> Self {
        Self {
            species: "Unknown".to_string(),
            nickname: None,
            level: 100,
            gender: None,
            shiny: false,
        }
    }
}

/// Pokemon state during battle (changes as battle progresses)
#[derive(Debug, Clone)]
pub struct PokemonState {
    /// Core identity
    pub identity: PokemonIdentity,

    // === HP ===
    /// Current HP (percentage for opponent, exact value for our Pokemon)
    pub hp_current: u32,

    /// Maximum HP (only known for our Pokemon)
    pub hp_max: Option<u32>,

    // === Status ===
    /// Non-volatile status condition
    pub status: Option<Status>,

    /// Whether this Pokemon has fainted
    pub fainted: bool,

    /// Whether this Pokemon is currently active on the field
    pub active: bool,

    // === Combat state (cleared on switch) ===
    /// Stat stage modifiers
    pub boosts: StatStages,

    /// Active volatile conditions
    pub volatiles: HashSet<Volatile>,

    // === Type tracking ===
    /// Original types from species
    pub base_types: Vec<Type>,

    /// Current types (may change via Forest's Curse, Soak, etc.)
    pub current_types: Vec<Type>,

    /// Tera type (if terastallized)
    pub tera_type: Option<Type>,

    /// Whether currently terastallized
    pub terastallized: bool,

    // === Revealed information ===
    /// Moves that have been revealed
    pub known_moves: Vec<String>,

    /// Ability that has been revealed
    pub known_ability: Option<String>,

    /// Item that has been revealed
    pub known_item: Option<String>,

    /// Whether the item has been consumed
    pub item_consumed: bool,

    // === Special states ===
    /// Species this Pokemon has transformed into
    pub transformed: Option<String>,

    /// Whether currently Dynamaxed
    pub dynamaxed: bool,

    /// Whether has mega evolved this battle
    pub mega_evolved: bool,
}

impl PokemonState {
    /// Create a new Pokemon state
    pub fn new(species: impl Into<String>, level: u8) -> Self {
        Self {
            identity: PokemonIdentity::new(species, level),
            hp_current: 100,
            hp_max: None,
            status: None,
            fainted: false,
            active: false,
            boosts: StatStages::new(),
            volatiles: HashSet::new(),
            base_types: Vec::new(),
            current_types: Vec::new(),
            tera_type: None,
            terastallized: false,
            known_moves: Vec::new(),
            known_ability: None,
            known_item: None,
            item_consumed: false,
            transformed: None,
            dynamaxed: false,
            mega_evolved: false,
        }
    }

    /// Create from protocol PokemonDetails
    pub fn from_protocol(details: &PokemonDetails) -> Self {
        let mut state = Self::new(&details.species, details.level.unwrap_or(100));
        state.identity = PokemonIdentity::from_protocol(details);

        // Parse tera type if present
        if let Some(ref tera_str) = details.tera_type {
            state.tera_type = Type::from_protocol(tera_str);
        }

        state
    }

    /// Create from protocol PokemonDetails with a nickname
    pub fn from_protocol_with_name(details: &PokemonDetails, name: &str) -> Self {
        let mut state = Self::from_protocol(details);
        if name != details.species {
            state.identity.nickname = Some(name.to_string());
        }
        state
    }

    /// Get HP as percentage (0-100)
    pub fn hp_percent(&self) -> u32 {
        if let Some(max) = self.hp_max {
            if max == 0 {
                return 0;
            }
            (self.hp_current * 100) / max
        } else {
            // For opponent Pokemon, hp_current IS the percentage
            self.hp_current
        }
    }

    /// Get display name (nickname or species)
    pub fn name(&self) -> &str {
        self.identity.name()
    }

    /// Check for a volatile condition
    pub fn has_volatile(&self, v: &Volatile) -> bool {
        self.volatiles.contains(v)
    }

    /// Add a volatile condition
    pub fn add_volatile(&mut self, v: Volatile) {
        self.volatiles.insert(v);
    }

    /// Remove a volatile condition
    pub fn remove_volatile(&mut self, v: &Volatile) -> bool {
        self.volatiles.remove(v)
    }

    /// Clear all volatiles
    pub fn clear_volatiles(&mut self) {
        self.volatiles.clear();
    }

    /// Record a revealed move
    pub fn record_move(&mut self, move_name: &str) {
        let move_name = move_name.to_string();
        if !self.known_moves.contains(&move_name) {
            self.known_moves.push(move_name);
        }
    }

    /// Record a revealed ability
    pub fn record_ability(&mut self, ability: &str) {
        self.known_ability = Some(ability.to_string());
    }

    /// Record a revealed item
    pub fn record_item(&mut self, item: &str) {
        self.known_item = Some(item.to_string());
        self.item_consumed = false;
    }

    /// Mark item as consumed
    pub fn consume_item(&mut self) {
        self.item_consumed = true;
    }

    /// Apply HP and status from protocol HpStatus
    pub fn apply_hp_status(&mut self, hp_status: &HpStatus) {
        self.hp_current = hp_status.current;
        if let Some(max) = hp_status.max {
            self.hp_max = Some(max);
        }

        // Parse status from protocol
        if let Some(ref status_str) = hp_status.status {
            if status_str == "fnt" {
                self.fainted = true;
                self.status = None;
            } else {
                self.status = Status::from_protocol(status_str);
            }
        } else {
            // No status in the hp_status, but don't clear existing status
            // unless we have full HP info (from request)
        }
    }

    /// Called when this Pokemon switches out
    pub fn on_switch_out(&mut self) {
        self.active = false;
        self.boosts.clear();
        self.volatiles.clear();
        self.dynamaxed = false;

        // Reset types to base types
        self.current_types = self.base_types.clone();
        self.terastallized = false;
    }

    /// Called when this Pokemon switches in
    pub fn on_switch_in(&mut self) {
        self.active = true;
    }

    /// Check if Pokemon is alive (not fainted)
    pub fn is_alive(&self) -> bool {
        !self.fainted && self.hp_current > 0
    }

    /// Check if Pokemon can be switched to
    pub fn can_switch_to(&self) -> bool {
        self.is_alive() && !self.active
    }

    /// Get current types (considering terastallization)
    pub fn get_types(&self) -> &[Type] {
        if self.terastallized {
            // When terastallized, only has the tera type for STAB/weakness purposes
            // This is a simplification - actual mechanics are more complex
            if let Some(ref _tera) = self.tera_type {
                // In practice, the current_types should be updated when terastallizing
                return &self.current_types;
            }
        }
        &self.current_types
    }

    /// Check if Pokemon has a specific type
    pub fn has_type(&self, t: Type) -> bool {
        self.current_types.contains(&t)
    }

    /// Set types (for forme changes, Transform, etc.)
    pub fn set_types(&mut self, types: Vec<Type>) {
        self.current_types = types;
    }

    /// Add a type (Forest's Curse, Trick-or-Treat)
    pub fn add_type(&mut self, t: Type) {
        if !self.current_types.contains(&t) {
            self.current_types.push(t);
        }
    }
}

impl Default for PokemonState {
    fn default() -> Self {
        Self {
            identity: PokemonIdentity::default(),
            hp_current: 100,
            hp_max: None,
            status: None,
            fainted: false,
            active: false,
            boosts: StatStages::new(),
            volatiles: HashSet::new(),
            base_types: Vec::new(),
            current_types: Vec::new(),
            tera_type: None,
            terastallized: false,
            known_moves: Vec::new(),
            known_ability: None,
            known_item: None,
            item_consumed: false,
            transformed: None,
            dynamaxed: false,
            mega_evolved: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pokemon_identity_new() {
        let ident = PokemonIdentity::new("Pikachu", 50);
        assert_eq!(ident.species, "Pikachu");
        assert_eq!(ident.level, 50);
        assert!(ident.nickname.is_none());
        assert_eq!(ident.name(), "Pikachu");
    }

    #[test]
    fn test_pokemon_identity_with_nickname() {
        let mut ident = PokemonIdentity::new("Pikachu", 50);
        ident.nickname = Some("Sparky".to_string());
        assert_eq!(ident.name(), "Sparky");
    }

    #[test]
    fn test_pokemon_state_new() {
        let state = PokemonState::new("Charizard", 100);
        assert_eq!(state.identity.species, "Charizard");
        assert_eq!(state.hp_current, 100);
        assert!(!state.fainted);
        assert!(!state.active);
        assert!(state.boosts.is_clear());
    }

    #[test]
    fn test_pokemon_state_hp_percent() {
        let mut state = PokemonState::new("Test", 100);

        // Without max HP (opponent), hp_current is the percentage
        state.hp_current = 75;
        assert_eq!(state.hp_percent(), 75);

        // With max HP (our Pokemon)
        state.hp_current = 150;
        state.hp_max = Some(200);
        assert_eq!(state.hp_percent(), 75);
    }

    #[test]
    fn test_pokemon_state_volatiles() {
        let mut state = PokemonState::new("Test", 100);

        state.add_volatile(Volatile::Confusion);
        assert!(state.has_volatile(&Volatile::Confusion));

        state.add_volatile(Volatile::Taunt);
        assert!(state.has_volatile(&Volatile::Taunt));

        state.remove_volatile(&Volatile::Confusion);
        assert!(!state.has_volatile(&Volatile::Confusion));
        assert!(state.has_volatile(&Volatile::Taunt));

        state.clear_volatiles();
        assert!(!state.has_volatile(&Volatile::Taunt));
    }

    #[test]
    fn test_pokemon_state_switch_out() {
        let mut state = PokemonState::new("Test", 100);
        state.active = true;
        state.boosts.atk = 2;
        state.add_volatile(Volatile::Confusion);
        state.dynamaxed = true;

        state.on_switch_out();

        assert!(!state.active);
        assert!(state.boosts.is_clear());
        assert!(state.volatiles.is_empty());
        assert!(!state.dynamaxed);
    }

    #[test]
    fn test_pokemon_state_record_move() {
        let mut state = PokemonState::new("Test", 100);

        state.record_move("Thunderbolt");
        state.record_move("Quick Attack");
        state.record_move("Thunderbolt"); // Duplicate

        assert_eq!(state.known_moves.len(), 2);
        assert!(state.known_moves.contains(&"Thunderbolt".to_string()));
        assert!(state.known_moves.contains(&"Quick Attack".to_string()));
    }

    #[test]
    fn test_pokemon_state_is_alive() {
        let mut state = PokemonState::new("Test", 100);
        assert!(state.is_alive());

        state.fainted = true;
        assert!(!state.is_alive());

        state.fainted = false;
        state.hp_current = 0;
        assert!(!state.is_alive());
    }

    #[test]
    fn test_pokemon_state_can_switch_to() {
        let mut state = PokemonState::new("Test", 100);
        state.hp_current = 100;

        assert!(state.can_switch_to());

        state.active = true;
        assert!(!state.can_switch_to());

        state.active = false;
        state.fainted = true;
        assert!(!state.can_switch_to());
    }

    #[test]
    fn test_pokemon_state_apply_hp_status() {
        let mut state = PokemonState::new("Test", 100);

        let hp_status = HpStatus {
            current: 75,
            max: Some(100),
            status: Some("par".to_string()),
        };

        state.apply_hp_status(&hp_status);
        assert_eq!(state.hp_current, 75);
        assert_eq!(state.hp_max, Some(100));
        assert_eq!(state.status, Some(Status::Paralysis));

        // Test fainted
        let faint_status = HpStatus {
            current: 0,
            max: None,
            status: Some("fnt".to_string()),
        };

        state.apply_hp_status(&faint_status);
        assert!(state.fainted);
        assert!(state.status.is_none());
    }
}
