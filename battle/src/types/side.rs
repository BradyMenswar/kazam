//! Side (player) state

use std::collections::HashMap;

use kazam_protocol::Player;

use super::conditions::{SideCondition, SideConditionState};
use super::pokemon::PokemonState;

/// One player's side of the battle
#[derive(Debug, Clone)]
pub struct SideState {
    /// Player identifier (P1, P2, etc.)
    pub player: Player,

    /// Player's username
    pub username: String,

    /// Pokemon on this side (party order)
    pub pokemon: Vec<PokemonState>,

    /// Currently active Pokemon indices
    /// For singles: [Some(idx)] or [None]
    /// For doubles: [Some(idx1), Some(idx2)] etc.
    pub active_indices: Vec<Option<usize>>,

    /// Side conditions (hazards, screens, etc.)
    pub conditions: HashMap<SideCondition, SideConditionState>,
}

impl SideState {
    /// Create a new side state
    pub fn new(player: Player, username: impl Into<String>) -> Self {
        Self {
            player,
            username: username.into(),
            pokemon: Vec::new(),
            active_indices: vec![None], // Default to singles
            conditions: HashMap::new(),
        }
    }

    /// Set the number of active slots (1 for singles, 2 for doubles, etc.)
    pub fn set_active_slots(&mut self, count: usize) {
        self.active_indices.resize(count, None);
    }

    /// Get the active Pokemon at a slot (0-indexed)
    pub fn active(&self, slot: usize) -> Option<&PokemonState> {
        self.active_indices
            .get(slot)
            .and_then(|idx| idx.as_ref())
            .and_then(|&idx| self.pokemon.get(idx))
    }

    /// Get the active Pokemon at a slot mutably
    pub fn active_mut(&mut self, slot: usize) -> Option<&mut PokemonState> {
        if let Some(Some(idx)) = self.active_indices.get(slot) {
            let idx = *idx;
            self.pokemon.get_mut(idx)
        } else {
            None
        }
    }

    /// Get the first active Pokemon (convenience for singles)
    pub fn active_pokemon(&self) -> Option<&PokemonState> {
        self.active(0)
    }

    /// Get the first active Pokemon mutably
    pub fn active_pokemon_mut(&mut self) -> Option<&mut PokemonState> {
        self.active_mut(0)
    }

    /// Iterate over all active Pokemon
    pub fn get_active(&self) -> impl Iterator<Item = &PokemonState> {
        self.active_indices
            .iter()
            .filter_map(|idx| idx.as_ref())
            .filter_map(|&idx| self.pokemon.get(idx))
    }

    /// Iterate over bench Pokemon (not active, not fainted)
    pub fn get_bench(&self) -> impl Iterator<Item = (usize, &PokemonState)> {
        let active_set: std::collections::HashSet<usize> = self
            .active_indices
            .iter()
            .filter_map(|idx| *idx)
            .collect();

        self.pokemon
            .iter()
            .enumerate()
            .filter(move |(idx, poke)| !active_set.contains(idx) && poke.is_alive())
    }

    /// Count non-fainted Pokemon
    pub fn alive_count(&self) -> usize {
        self.pokemon.iter().filter(|p| p.is_alive()).count()
    }

    /// Count fainted Pokemon
    pub fn fainted_count(&self) -> usize {
        self.pokemon.iter().filter(|p| p.fainted).count()
    }

    /// Find a Pokemon by name (nickname or species)
    pub fn find_pokemon(&self, name: &str) -> Option<usize> {
        self.pokemon
            .iter()
            .position(|p| p.name() == name || p.identity.species == name)
    }

    /// Find a Pokemon by name and get a mutable reference
    pub fn find_pokemon_mut(&mut self, name: &str) -> Option<&mut PokemonState> {
        self.pokemon
            .iter_mut()
            .find(|p| p.name() == name || p.identity.species == name)
    }

    /// Get a Pokemon by index
    pub fn get_pokemon(&self, index: usize) -> Option<&PokemonState> {
        self.pokemon.get(index)
    }

    /// Get a Pokemon by index mutably
    pub fn get_pokemon_mut(&mut self, index: usize) -> Option<&mut PokemonState> {
        self.pokemon.get_mut(index)
    }

    /// Check if side has a condition
    pub fn has_condition(&self, cond: SideCondition) -> bool {
        self.conditions.contains_key(&cond)
    }

    /// Get layers for a condition (0 if not present)
    pub fn condition_layers(&self, cond: SideCondition) -> u8 {
        self.conditions.get(&cond).map_or(0, |s| s.layers)
    }

    /// Add a side condition
    /// Returns true if the condition was added (false if already at max layers)
    pub fn add_condition(&mut self, cond: SideCondition) -> bool {
        if let Some(state) = self.conditions.get_mut(&cond) {
            // Already have this condition, try to add a layer
            state.add_layer(cond)
        } else {
            // New condition
            self.conditions.insert(cond, SideConditionState::new());
            true
        }
    }

    /// Remove a side condition
    pub fn remove_condition(&mut self, cond: SideCondition) -> bool {
        self.conditions.remove(&cond).is_some()
    }

    /// Clear all side conditions
    pub fn clear_conditions(&mut self) {
        self.conditions.clear();
    }

    /// Check if all Pokemon have fainted
    pub fn all_fainted(&self) -> bool {
        !self.pokemon.is_empty() && self.pokemon.iter().all(|p| p.fainted)
    }

    /// Set the active Pokemon at a slot
    pub fn set_active(&mut self, slot: usize, pokemon_index: Option<usize>) {
        if slot < self.active_indices.len() {
            // Switch out old active Pokemon if any
            if let Some(old_idx) = self.active_indices[slot] {
                if let Some(old_poke) = self.pokemon.get_mut(old_idx) {
                    old_poke.on_switch_out();
                }
            }

            self.active_indices[slot] = pokemon_index;

            // Switch in new Pokemon
            if let Some(idx) = pokemon_index {
                if let Some(new_poke) = self.pokemon.get_mut(idx) {
                    new_poke.on_switch_in();
                }
            }
        }
    }

    /// Find the active slot for a Pokemon index
    pub fn find_active_slot(&self, pokemon_index: usize) -> Option<usize> {
        self.active_indices
            .iter()
            .position(|idx| *idx == Some(pokemon_index))
    }

    /// Check if any hazards are set
    pub fn has_hazards(&self) -> bool {
        self.conditions.keys().any(|c| c.is_hazard())
    }

    /// Check if any screens are active
    pub fn has_screens(&self) -> bool {
        self.conditions.keys().any(|c| c.is_screen())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_side() -> SideState {
        let mut side = SideState::new(Player::P1, "TestUser");

        // Add some Pokemon
        let mut poke1 = PokemonState::new("Pikachu", 50);
        poke1.hp_current = 100;

        let mut poke2 = PokemonState::new("Charizard", 50);
        poke2.hp_current = 100;

        let mut poke3 = PokemonState::new("Blastoise", 50);
        poke3.hp_current = 0;
        poke3.fainted = true;

        side.pokemon.push(poke1);
        side.pokemon.push(poke2);
        side.pokemon.push(poke3);

        side
    }

    #[test]
    fn test_new_side() {
        let side = SideState::new(Player::P1, "Alice");
        assert_eq!(side.player, Player::P1);
        assert_eq!(side.username, "Alice");
        assert!(side.pokemon.is_empty());
        assert_eq!(side.active_indices.len(), 1);
    }

    #[test]
    fn test_set_active_slots() {
        let mut side = SideState::new(Player::P1, "Test");
        assert_eq!(side.active_indices.len(), 1);

        side.set_active_slots(2);
        assert_eq!(side.active_indices.len(), 2);

        side.set_active_slots(3);
        assert_eq!(side.active_indices.len(), 3);
    }

    #[test]
    fn test_active_pokemon() {
        let mut side = create_test_side();
        side.active_indices[0] = Some(0);

        let active = side.active_pokemon().unwrap();
        assert_eq!(active.identity.species, "Pikachu");
    }

    #[test]
    fn test_get_bench() {
        let mut side = create_test_side();
        side.active_indices[0] = Some(0); // Pikachu is active

        let bench: Vec<_> = side.get_bench().collect();
        // Should only have Charizard (Blastoise is fainted)
        assert_eq!(bench.len(), 1);
        assert_eq!(bench[0].1.identity.species, "Charizard");
    }

    #[test]
    fn test_alive_count() {
        let side = create_test_side();
        assert_eq!(side.alive_count(), 2); // Pikachu and Charizard
        assert_eq!(side.fainted_count(), 1); // Blastoise
    }

    #[test]
    fn test_find_pokemon() {
        let side = create_test_side();
        assert_eq!(side.find_pokemon("Pikachu"), Some(0));
        assert_eq!(side.find_pokemon("Charizard"), Some(1));
        assert_eq!(side.find_pokemon("Unknown"), None);
    }

    #[test]
    fn test_side_conditions() {
        let mut side = SideState::new(Player::P1, "Test");

        // Add Stealth Rock
        assert!(side.add_condition(SideCondition::StealthRock));
        assert!(side.has_condition(SideCondition::StealthRock));
        assert_eq!(side.condition_layers(SideCondition::StealthRock), 1);

        // Can't add more (max 1)
        assert!(!side.add_condition(SideCondition::StealthRock));
        assert_eq!(side.condition_layers(SideCondition::StealthRock), 1);

        // Add Spikes (stackable)
        assert!(side.add_condition(SideCondition::Spikes));
        assert_eq!(side.condition_layers(SideCondition::Spikes), 1);
        assert!(side.add_condition(SideCondition::Spikes));
        assert_eq!(side.condition_layers(SideCondition::Spikes), 2);
        assert!(side.add_condition(SideCondition::Spikes));
        assert_eq!(side.condition_layers(SideCondition::Spikes), 3);
        assert!(!side.add_condition(SideCondition::Spikes)); // Max 3
        assert_eq!(side.condition_layers(SideCondition::Spikes), 3);

        // Remove condition
        assert!(side.remove_condition(SideCondition::Spikes));
        assert!(!side.has_condition(SideCondition::Spikes));
    }

    #[test]
    fn test_all_fainted() {
        let mut side = create_test_side();
        assert!(!side.all_fainted());

        // Faint all Pokemon
        for poke in &mut side.pokemon {
            poke.fainted = true;
            poke.hp_current = 0;
        }
        assert!(side.all_fainted());
    }

    #[test]
    fn test_set_active() {
        let mut side = create_test_side();

        // Set Pikachu as active
        side.set_active(0, Some(0));
        assert!(side.pokemon[0].active);

        // Switch to Charizard
        side.set_active(0, Some(1));
        assert!(!side.pokemon[0].active); // Pikachu switched out
        assert!(side.pokemon[1].active); // Charizard switched in
    }

    #[test]
    fn test_has_hazards_and_screens() {
        let mut side = SideState::new(Player::P1, "Test");

        assert!(!side.has_hazards());
        assert!(!side.has_screens());

        side.add_condition(SideCondition::StealthRock);
        assert!(side.has_hazards());
        assert!(!side.has_screens());

        side.add_condition(SideCondition::Reflect);
        assert!(side.has_hazards());
        assert!(side.has_screens());
    }
}
