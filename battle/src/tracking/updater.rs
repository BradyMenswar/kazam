//! Update logic for processing ServerMessage into battle state

use kazam_protocol::{BattleRequest, Pokemon, PokemonDetails, ServerMessage};

use super::battle::{position_to_slot, TrackedBattle};
use crate::types::{
    PokemonState, SideCondition, Status, Volatile, Weather,
};

impl TrackedBattle {
    /// Update battle state from a server message
    pub fn update(&mut self, msg: &ServerMessage) {
        match msg {
            // === Battle Initialization ===
            ServerMessage::BattlePlayer {
                player,
                username,
                avatar: _,
                rating: _,
            } => {
                self.get_or_create_side(*player, username);
            }

            ServerMessage::TeamSize { player, size: _ } => {
                // Side should already exist from BattlePlayer
                // Team size is informational, we discover actual team from switches
                let _ = self.get_side(*player);
            }

            ServerMessage::GameType(game_type) => {
                self.set_game_type(*game_type);
            }

            ServerMessage::Gen(generation) => {
                self.generation = *generation;
            }

            ServerMessage::Tier(tier) => {
                self.tier = tier.clone();
            }

            ServerMessage::Turn(turn) => {
                self.turn = *turn;
            }

            // === Major Actions ===
            ServerMessage::Switch {
                pokemon,
                details,
                hp_status,
            } => {
                self.handle_switch(pokemon, details, hp_status.as_ref(), false);
            }

            ServerMessage::Drag {
                pokemon,
                details,
                hp_status,
            } => {
                self.handle_switch(pokemon, details, hp_status.as_ref(), true);
            }

            ServerMessage::Faint(pokemon) => {
                self.handle_faint(pokemon);
            }

            ServerMessage::Move {
                pokemon,
                move_name,
                target: _,
                miss: _,
                still: _,
                anim: _,
            } => {
                // Record the move as known
                if let Some(poke) = self.find_pokemon_mut(pokemon) {
                    poke.record_move(move_name);
                }
            }

            // === HP Changes ===
            ServerMessage::Damage { pokemon, hp_status } => {
                if let (Some(poke), Some(hp)) = (self.find_pokemon_mut(pokemon), hp_status) {
                    poke.apply_hp_status(hp);
                }
            }

            ServerMessage::Heal { pokemon, hp_status } => {
                if let (Some(poke), Some(hp)) = (self.find_pokemon_mut(pokemon), hp_status) {
                    poke.apply_hp_status(hp);
                }
            }

            ServerMessage::SetHp { pokemon, hp_status } => {
                if let (Some(poke), Some(hp)) = (self.find_pokemon_mut(pokemon), hp_status) {
                    poke.apply_hp_status(hp);
                }
            }

            // === Status ===
            ServerMessage::Status { pokemon, status } => {
                if let Some(poke) = self.find_pokemon_mut(pokemon) {
                    poke.status = Status::from_protocol(status);
                }
            }

            ServerMessage::CureStatus { pokemon, status: _ } => {
                if let Some(poke) = self.find_pokemon_mut(pokemon) {
                    poke.status = None;
                }
            }

            ServerMessage::CureTeam(pokemon) => {
                // Cure status for entire team
                if let Some(side) = self.get_side_mut(pokemon.player) {
                    for poke in &mut side.pokemon {
                        poke.status = None;
                    }
                }
            }

            // === Boosts ===
            ServerMessage::Boost {
                pokemon,
                stat,
                amount,
            } => {
                if let Some(poke) = self.find_pokemon_mut(pokemon) {
                    poke.boosts.boost(*stat, *amount);
                }
            }

            ServerMessage::Unboost {
                pokemon,
                stat,
                amount,
            } => {
                if let Some(poke) = self.find_pokemon_mut(pokemon) {
                    poke.boosts.unboost(*stat, *amount);
                }
            }

            ServerMessage::SetBoost {
                pokemon,
                stat,
                amount,
            } => {
                if let Some(poke) = self.find_pokemon_mut(pokemon) {
                    poke.boosts.set(*stat, *amount);
                }
            }

            ServerMessage::ClearBoost(pokemon) => {
                if let Some(poke) = self.find_pokemon_mut(pokemon) {
                    poke.boosts.clear();
                }
            }

            ServerMessage::ClearAllBoost => {
                // Clear boosts for all active Pokemon
                for side in self.sides.iter_mut().flatten() {
                    for idx in &side.active_indices {
                        if let Some(idx) = idx
                            && let Some(poke) = side.pokemon.get_mut(*idx) {
                                poke.boosts.clear();
                            }
                    }
                }
            }

            ServerMessage::InvertBoost(pokemon) => {
                if let Some(poke) = self.find_pokemon_mut(pokemon) {
                    poke.boosts.invert();
                }
            }

            ServerMessage::ClearPositiveBoost {
                target,
                source: _,
                effect: _,
            } => {
                if let Some(poke) = self.find_pokemon_mut(target) {
                    poke.boosts.clear_positive();
                }
            }

            ServerMessage::ClearNegativeBoost(pokemon) => {
                if let Some(poke) = self.find_pokemon_mut(pokemon) {
                    poke.boosts.clear_negative();
                }
            }

            ServerMessage::CopyBoost { source, target } => {
                // Copy boosts from source to target
                let source_boosts = self
                    .find_pokemon(source)
                    .map(|p| p.boosts.clone());

                if let (Some(boosts), Some(target_poke)) =
                    (source_boosts, self.find_pokemon_mut(target))
                {
                    target_poke.boosts.copy_from(&boosts);
                }
            }

            ServerMessage::SwapBoost {
                source,
                target,
                stats,
            } => {
                // Swap specific stat boosts between source and target
                let source_boosts = self.find_pokemon(source).map(|p| p.boosts.clone());
                let target_boosts = self.find_pokemon(target).map(|p| p.boosts.clone());

                if let (Some(src_boosts), Some(tgt_boosts)) = (source_boosts, target_boosts) {
                    if let Some(src_poke) = self.find_pokemon_mut(source) {
                        for stat in stats {
                            src_poke.boosts.set(*stat, tgt_boosts.get(*stat));
                        }
                    }
                    if let Some(tgt_poke) = self.find_pokemon_mut(target) {
                        for stat in stats {
                            tgt_poke.boosts.set(*stat, src_boosts.get(*stat));
                        }
                    }
                }
            }

            // === Volatiles ===
            ServerMessage::VolatileStart { pokemon, effect } => {
                if let Some(poke) = self.find_pokemon_mut(pokemon) {
                    let volatile = Volatile::from_protocol(effect);
                    poke.add_volatile(volatile);
                }
            }

            ServerMessage::VolatileEnd { pokemon, effect } => {
                if let Some(poke) = self.find_pokemon_mut(pokemon) {
                    let volatile = Volatile::from_protocol(effect);
                    poke.remove_volatile(&volatile);
                }
            }

            // === Field Conditions ===
            ServerMessage::Weather { weather, upkeep } => {
                if !upkeep {
                    // Only update on initial weather set, not upkeep messages
                    if weather == "none" || weather.is_empty() {
                        self.field.weather = None;
                    } else {
                        self.field.weather = Weather::from_protocol(weather);
                    }
                }
            }

            ServerMessage::FieldStart(condition) => {
                self.field.apply_field_start(condition);
            }

            ServerMessage::FieldEnd(condition) => {
                self.field.apply_field_end(condition);
            }

            // === Side Conditions ===
            ServerMessage::SideStart { side, condition } => {
                if let Some(side_state) = self.get_side_mut(side.player)
                    && let Some(cond) = SideCondition::from_protocol(condition) {
                        side_state.add_condition(cond);
                    }
            }

            ServerMessage::SideEnd { side, condition } => {
                if let Some(side_state) = self.get_side_mut(side.player)
                    && let Some(cond) = SideCondition::from_protocol(condition) {
                        side_state.remove_condition(cond);
                    }
            }

            ServerMessage::SwapSideConditions => {
                // Swap side conditions between P1 and P2 (Court Change)
                let p1_conditions = self.get_side(kazam_protocol::Player::P1)
                    .map(|s| s.conditions.clone());
                let p2_conditions = self.get_side(kazam_protocol::Player::P2)
                    .map(|s| s.conditions.clone());

                if let (Some(c1), Some(c2)) = (p1_conditions, p2_conditions) {
                    if let Some(s1) = self.get_side_mut(kazam_protocol::Player::P1) {
                        s1.conditions = c2;
                    }
                    if let Some(s2) = self.get_side_mut(kazam_protocol::Player::P2) {
                        s2.conditions = c1;
                    }
                }
            }

            // === Items and Abilities ===
            ServerMessage::Item {
                pokemon,
                item,
                from: _,
            } => {
                if let Some(poke) = self.find_pokemon_mut(pokemon) {
                    poke.record_item(item);
                }
            }

            ServerMessage::EndItem {
                pokemon,
                item: _,
                from: _,
                eat: _,
            } => {
                if let Some(poke) = self.find_pokemon_mut(pokemon) {
                    poke.consume_item();
                }
            }

            ServerMessage::Ability {
                pokemon,
                ability,
                from: _,
            } => {
                if let Some(poke) = self.find_pokemon_mut(pokemon) {
                    poke.record_ability(ability);
                }
            }

            ServerMessage::EndAbility(pokemon) => {
                // Ability suppressed (Gastro Acid, etc.)
                if let Some(poke) = self.find_pokemon_mut(pokemon) {
                    poke.add_volatile(Volatile::GastroAcid);
                }
            }

            // === Transformations ===
            ServerMessage::Transform { pokemon, species } => {
                if let Some(poke) = self.find_pokemon_mut(pokemon) {
                    poke.transformed = Some(species.clone());
                    poke.add_volatile(Volatile::Transformed);
                }
            }

            ServerMessage::Mega { pokemon, megastone: _ } => {
                if let Some(poke) = self.find_pokemon_mut(pokemon) {
                    poke.mega_evolved = true;
                }
            }

            ServerMessage::DetailsChange {
                pokemon,
                details,
                hp_status,
            } => {
                // Forme change that persists (Mega Evolution, etc.)
                if let Some(poke) = self.find_pokemon_mut(pokemon) {
                    poke.identity.species = details.species.clone();
                    if let Some(hp) = hp_status {
                        poke.apply_hp_status(hp);
                    }
                }
            }

            ServerMessage::FormeChange {
                pokemon,
                species,
                hp_status,
            } => {
                // Temporary forme change
                if let Some(poke) = self.find_pokemon_mut(pokemon) {
                    // Store current species if transforming
                    poke.identity.species = species.clone();
                    if let Some(hp) = hp_status {
                        poke.apply_hp_status(hp);
                    }
                }
            }

            // === Battle End ===
            ServerMessage::Win(winner) => {
                self.ended = true;
                self.winner = Some(winner.clone());
            }

            ServerMessage::Tie => {
                self.ended = true;
                self.tie = true;
            }

            // === Ignored Messages (informational only) ===
            ServerMessage::Crit(_)
            | ServerMessage::SuperEffective(_)
            | ServerMessage::Resisted(_)
            | ServerMessage::Immune(_)
            | ServerMessage::Miss { .. }
            | ServerMessage::Fail { .. }
            | ServerMessage::Block { .. }
            | ServerMessage::NoTarget(_)
            | ServerMessage::Cant { .. }
            | ServerMessage::Upkeep
            | ServerMessage::Request(_)
            | ServerMessage::Inactive(_)
            | ServerMessage::InactiveOff(_)
            | ServerMessage::BattleStart
            | ServerMessage::ClearPoke
            | ServerMessage::Poke { .. }
            | ServerMessage::TeamPreview(_)
            | ServerMessage::Rated(_)
            | ServerMessage::Rule(_)
            | ServerMessage::Primal(_)
            | ServerMessage::Swap { .. }
            | ServerMessage::Replace { .. } => {
                // These don't affect tracked state
            }

            // === Non-battle messages ===
            _ => {
                // Ignore non-battle messages
            }
        }
    }

    /// Update battle state from a BattleRequest (provides full team info for our side)
    pub fn update_from_request(&mut self, request: &BattleRequest) {
        // Extract perspective from side info
        if let Some(ref side_info) = request.side {
            // Parse player from side id (e.g., "p1" -> Player::P1)
            if let Some(player) = kazam_protocol::Player::parse(&side_info.id) {
                self.set_perspective(player);

                // Get or create our side
                let side = self.get_or_create_side(player, &side_info.name);

                // Sync Pokemon from request (has full info)
                for (i, req_poke) in side_info.pokemon.iter().enumerate() {
                    if i >= side.pokemon.len() {
                        // Add new Pokemon from request
                        let mut poke = PokemonState::new(&req_poke.details, 100);

                        // Parse details
                        let details = PokemonDetails::parse(&req_poke.details);
                        poke.identity.species = details.species;
                        poke.identity.level = details.level.unwrap_or(100);
                        poke.identity.gender = details.gender;
                        poke.identity.shiny = details.shiny;

                        // Parse nickname from ident
                        if let Some(name) = req_poke.ident.split(": ").nth(1)
                            && name != poke.identity.species {
                                poke.identity.nickname = Some(name.to_string());
                            }

                        // Full info from request
                        poke.known_moves = req_poke.moves.clone();
                        poke.known_ability = Some(req_poke.ability.clone());
                        poke.known_item = if req_poke.item.is_empty() {
                            None
                        } else {
                            Some(req_poke.item.clone())
                        };
                        poke.active = req_poke.active;

                        // Parse HP from condition
                        if let Some((current, max)) = req_poke.hp() {
                            poke.hp_current = current;
                            poke.hp_max = Some(max);
                        }

                        // Parse status from condition
                        if let Some(status_str) = req_poke.status() {
                            poke.status = Status::from_protocol(status_str);
                            if status_str == "fnt" {
                                poke.fainted = true;
                            }
                        }

                        side.pokemon.push(poke);
                    } else {
                        // Update existing Pokemon with full info
                        let poke = &mut side.pokemon[i];
                        poke.known_moves = req_poke.moves.clone();
                        poke.known_ability = Some(req_poke.ability.clone());
                        poke.known_item = if req_poke.item.is_empty() {
                            None
                        } else {
                            Some(req_poke.item.clone())
                        };
                        poke.active = req_poke.active;

                        if let Some((current, max)) = req_poke.hp() {
                            poke.hp_current = current;
                            poke.hp_max = Some(max);
                        }

                        if let Some(status_str) = req_poke.status() {
                            if status_str == "fnt" {
                                poke.fainted = true;
                                poke.status = None;
                            } else {
                                poke.status = Status::from_protocol(status_str);
                            }
                        } else {
                            poke.status = None;
                            poke.fainted = poke.hp_current == 0;
                        }
                    }
                }
            }
        }
    }

    /// Handle a switch (or drag) message
    fn handle_switch(
        &mut self,
        pokemon: &Pokemon,
        details: &PokemonDetails,
        hp_status: Option<&kazam_protocol::HpStatus>,
        _is_drag: bool,
    ) {
        let slot = pokemon.position.map(position_to_slot).unwrap_or(0);

        let side = self.get_or_create_side(pokemon.player, "");

        // Find existing Pokemon or create new one
        let poke_idx = side
            .find_pokemon(&pokemon.name)
            .unwrap_or_else(|| {
                // New Pokemon
                let poke = PokemonState::from_protocol_with_name(details, &pokemon.name);
                side.pokemon.push(poke);
                side.pokemon.len() - 1
            });

        // Update the Pokemon's details (may have changed forme)
        let poke = &mut side.pokemon[poke_idx];
        poke.identity.species = details.species.clone();
        poke.identity.level = details.level.unwrap_or(100);
        poke.identity.gender = details.gender;
        poke.identity.shiny = details.shiny;

        if let Some(hp) = hp_status {
            poke.apply_hp_status(hp);
        }

        // Update active slot
        side.set_active(slot, Some(poke_idx));
    }

    /// Handle a faint message
    fn handle_faint(&mut self, pokemon: &Pokemon) {
        if let Some(poke) = self.find_pokemon_mut(pokemon) {
            poke.fainted = true;
            poke.hp_current = 0;
            poke.active = false;
        }

        // Clear from active slot
        if let Some(side) = self.get_side_mut(pokemon.player)
            && let Some(slot) = pokemon.position.map(position_to_slot) {
                side.active_indices[slot] = None;
            }
    }

    /// Find a Pokemon by protocol identifier (immutable)
    fn find_pokemon(&self, pokemon: &Pokemon) -> Option<&PokemonState> {
        self.get_side(pokemon.player)?
            .pokemon
            .iter()
            .find(|p| p.name() == pokemon.name || p.identity.species == pokemon.name)
    }

    /// Find a Pokemon by protocol identifier (mutable)
    fn find_pokemon_mut(&mut self, pokemon: &Pokemon) -> Option<&mut PokemonState> {
        self.get_side_mut(pokemon.player)?
            .find_pokemon_mut(&pokemon.name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kazam_protocol::{GameType, HpStatus, Player, Stat};

    fn create_test_pokemon(name: &str, _level: u8) -> Pokemon {
        Pokemon {
            player: Player::P1,
            position: Some('a'),
            name: name.to_string(),
        }
    }

    fn create_test_details(species: &str) -> PokemonDetails {
        PokemonDetails {
            species: species.to_string(),
            level: Some(50),
            gender: None,
            shiny: false,
            tera_type: None,
        }
    }

    #[test]
    fn test_update_battle_player() {
        let mut battle = TrackedBattle::new();

        battle.update(&ServerMessage::BattlePlayer {
            player: Player::P1,
            username: "Alice".to_string(),
            avatar: "1".to_string(),
            rating: Some(1500),
        });

        assert!(battle.has_side(Player::P1));
        assert_eq!(battle.get_side(Player::P1).unwrap().username, "Alice");
    }

    #[test]
    fn test_update_game_type() {
        let mut battle = TrackedBattle::new();
        battle.get_or_create_side(Player::P1, "Test");

        battle.update(&ServerMessage::GameType(GameType::Doubles));

        assert_eq!(battle.game_type, Some(GameType::Doubles));
        assert_eq!(
            battle.get_side(Player::P1).unwrap().active_indices.len(),
            2
        );
    }

    #[test]
    fn test_update_switch() {
        let mut battle = TrackedBattle::new();
        battle.get_or_create_side(Player::P1, "Test");

        battle.update(&ServerMessage::Switch {
            pokemon: create_test_pokemon("Pikachu", 50),
            details: create_test_details("Pikachu"),
            hp_status: Some(HpStatus {
                current: 100,
                max: Some(100),
                status: None,
            }),
        });

        let side = battle.get_side(Player::P1).unwrap();
        assert_eq!(side.pokemon.len(), 1);
        assert_eq!(side.pokemon[0].identity.species, "Pikachu");
        assert!(side.pokemon[0].active);
    }

    #[test]
    fn test_update_damage() {
        let mut battle = TrackedBattle::new();
        battle.get_or_create_side(Player::P1, "Test");

        // First switch in
        battle.update(&ServerMessage::Switch {
            pokemon: create_test_pokemon("Pikachu", 50),
            details: create_test_details("Pikachu"),
            hp_status: Some(HpStatus {
                current: 100,
                max: Some(100),
                status: None,
            }),
        });

        // Take damage
        battle.update(&ServerMessage::Damage {
            pokemon: create_test_pokemon("Pikachu", 50),
            hp_status: Some(HpStatus {
                current: 50,
                max: Some(100),
                status: None,
            }),
        });

        let poke = &battle.get_side(Player::P1).unwrap().pokemon[0];
        assert_eq!(poke.hp_current, 50);
    }

    #[test]
    fn test_update_boost() {
        let mut battle = TrackedBattle::new();
        battle.get_or_create_side(Player::P1, "Test");

        battle.update(&ServerMessage::Switch {
            pokemon: create_test_pokemon("Pikachu", 50),
            details: create_test_details("Pikachu"),
            hp_status: None,
        });

        battle.update(&ServerMessage::Boost {
            pokemon: create_test_pokemon("Pikachu", 50),
            stat: Stat::Atk,
            amount: 2,
        });

        let poke = &battle.get_side(Player::P1).unwrap().pokemon[0];
        assert_eq!(poke.boosts.atk, 2);
    }

    #[test]
    fn test_update_status() {
        let mut battle = TrackedBattle::new();
        battle.get_or_create_side(Player::P1, "Test");

        battle.update(&ServerMessage::Switch {
            pokemon: create_test_pokemon("Pikachu", 50),
            details: create_test_details("Pikachu"),
            hp_status: None,
        });

        battle.update(&ServerMessage::Status {
            pokemon: create_test_pokemon("Pikachu", 50),
            status: "par".to_string(),
        });

        let poke = &battle.get_side(Player::P1).unwrap().pokemon[0];
        assert_eq!(poke.status, Some(Status::Paralysis));

        battle.update(&ServerMessage::CureStatus {
            pokemon: create_test_pokemon("Pikachu", 50),
            status: "par".to_string(),
        });

        let poke = &battle.get_side(Player::P1).unwrap().pokemon[0];
        assert!(poke.status.is_none());
    }

    #[test]
    fn test_update_weather() {
        let mut battle = TrackedBattle::new();

        battle.update(&ServerMessage::Weather {
            weather: "SunnyDay".to_string(),
            upkeep: false,
        });

        assert_eq!(battle.field.weather, Some(Weather::Sun));

        // Upkeep messages shouldn't change weather
        battle.update(&ServerMessage::Weather {
            weather: "SunnyDay".to_string(),
            upkeep: true,
        });

        assert_eq!(battle.field.weather, Some(Weather::Sun));
    }

    #[test]
    fn test_update_faint() {
        let mut battle = TrackedBattle::new();
        battle.get_or_create_side(Player::P1, "Test");

        battle.update(&ServerMessage::Switch {
            pokemon: create_test_pokemon("Pikachu", 50),
            details: create_test_details("Pikachu"),
            hp_status: None,
        });

        battle.update(&ServerMessage::Faint(create_test_pokemon("Pikachu", 50)));

        let poke = &battle.get_side(Player::P1).unwrap().pokemon[0];
        assert!(poke.fainted);
        assert_eq!(poke.hp_current, 0);
    }

    #[test]
    fn test_update_win() {
        let mut battle = TrackedBattle::new();

        battle.update(&ServerMessage::Win("Alice".to_string()));

        assert!(battle.ended);
        assert_eq!(battle.winner, Some("Alice".to_string()));
    }
}
