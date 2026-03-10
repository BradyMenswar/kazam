//! Update logic for processing ServerMessage into battle state

use kazam_protocol::{BattleRequest, Pokemon, PokemonDetails, ServerFrame, ServerMessage};

use super::battle::{BattleKnowledge, TrackedBattle, position_to_slot};
use crate::types::{
    PokemonState, SideCondition, Status, Volatile, Weather,
};

impl TrackedBattle {
    /// Apply a single protocol message to the battle state.
    pub fn apply_message(&mut self, msg: &ServerMessage) {
        match msg {
            // === Battle Initialization ===
            ServerMessage::BattlePlayer {
                player,
                username,
                avatar: _,
                rating: _,
            } => {
                let side = self.get_or_create_side(*player, username);
                if side.username.is_empty() {
                    side.username = username.clone();
                }
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

    /// Apply a sequence of protocol messages to the battle state.
    pub fn apply_messages<'a, I>(&mut self, messages: I)
    where
        I: IntoIterator<Item = &'a ServerMessage>,
    {
        for message in messages {
            self.apply_message(message);
        }
    }

    /// Apply all protocol messages contained in a parsed frame.
    pub fn apply_frame(&mut self, frame: &ServerFrame) {
        self.apply_messages(frame.messages.iter());
    }

    /// Apply private request data for one player's view of the battle.
    ///
    /// This is an optional enrichment step used by live clients. Replay-style
    /// omniscient logs can skip it entirely.
    pub fn apply_request(&mut self, request: &BattleRequest) {
        // Extract perspective from side info
        if let Some(ref side_info) = request.side {
            // Parse player from side id (e.g., "p1" -> Player::P1)
            if let Some(player) = kazam_protocol::Player::parse(&side_info.id) {
                if self.knowledge() != BattleKnowledge::Omniscient {
                    self.set_knowledge(BattleKnowledge::Player(player));
                }
                if self.viewpoint().is_none() {
                    self.set_viewpoint(player);
                }

                // Get or create our side
                let side = self.get_or_create_side(player, &side_info.name);
                if side.username.is_empty() {
                    side.username = side_info.name.clone();
                }

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

    /// Backwards-compatible alias for `apply_message`.
    pub fn update(&mut self, msg: &ServerMessage) {
        self.apply_message(msg);
    }

    /// Backwards-compatible alias for `apply_request`.
    pub fn update_from_request(&mut self, request: &BattleRequest) {
        self.apply_request(request);
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
    use kazam_protocol::{GameType, HpStatus, Player, Stat, parse_server_message};

    use crate::{BattleKnowledge, SideCondition, Weather};

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

        battle.apply_message(&ServerMessage::BattlePlayer {
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

        battle.apply_message(&ServerMessage::GameType(GameType::Doubles));

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

        battle.apply_message(&ServerMessage::Switch {
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
        battle.apply_message(&ServerMessage::Switch {
            pokemon: create_test_pokemon("Pikachu", 50),
            details: create_test_details("Pikachu"),
            hp_status: Some(HpStatus {
                current: 100,
                max: Some(100),
                status: None,
            }),
        });

        // Take damage
        battle.apply_message(&ServerMessage::Damage {
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

        battle.apply_message(&ServerMessage::Switch {
            pokemon: create_test_pokemon("Pikachu", 50),
            details: create_test_details("Pikachu"),
            hp_status: None,
        });

        battle.apply_message(&ServerMessage::Boost {
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

        battle.apply_message(&ServerMessage::Switch {
            pokemon: create_test_pokemon("Pikachu", 50),
            details: create_test_details("Pikachu"),
            hp_status: None,
        });

        battle.apply_message(&ServerMessage::Status {
            pokemon: create_test_pokemon("Pikachu", 50),
            status: "par".to_string(),
        });

        let poke = &battle.get_side(Player::P1).unwrap().pokemon[0];
        assert_eq!(poke.status, Some(Status::Paralysis));

        battle.apply_message(&ServerMessage::CureStatus {
            pokemon: create_test_pokemon("Pikachu", 50),
            status: "par".to_string(),
        });

        let poke = &battle.get_side(Player::P1).unwrap().pokemon[0];
        assert!(poke.status.is_none());
    }

    #[test]
    fn test_update_weather() {
        let mut battle = TrackedBattle::new();

        battle.apply_message(&ServerMessage::Weather {
            weather: "SunnyDay".to_string(),
            upkeep: false,
        });

        assert_eq!(battle.field.weather, Some(Weather::Sun));

        // Upkeep messages shouldn't change weather
        battle.apply_message(&ServerMessage::Weather {
            weather: "SunnyDay".to_string(),
            upkeep: true,
        });

        assert_eq!(battle.field.weather, Some(Weather::Sun));
    }

    #[test]
    fn test_update_faint() {
        let mut battle = TrackedBattle::new();
        battle.get_or_create_side(Player::P1, "Test");

        battle.apply_message(&ServerMessage::Switch {
            pokemon: create_test_pokemon("Pikachu", 50),
            details: create_test_details("Pikachu"),
            hp_status: None,
        });

        battle.apply_message(&ServerMessage::Faint(create_test_pokemon("Pikachu", 50)));

        let poke = &battle.get_side(Player::P1).unwrap().pokemon[0];
        assert!(poke.fainted);
        assert_eq!(poke.hp_current, 0);
    }

    #[test]
    fn test_update_win() {
        let mut battle = TrackedBattle::new();

        battle.apply_message(&ServerMessage::Win("Alice".to_string()));

        assert!(battle.ended);
        assert_eq!(battle.winner, Some("Alice".to_string()));
    }

    #[test]
    fn test_apply_request_promotes_player_knowledge() {
        let json = serde_json::json!({
            "rqid": 7,
            "side": {
                "name": "Alice",
                "id": "p1",
                "pokemon": [{
                    "ident": "p1: Pikachu",
                    "details": "Pikachu, L50",
                    "condition": "100/100",
                    "active": true,
                    "moves": ["thunderbolt", "surf"],
                    "ability": "Static",
                    "item": "Light Ball"
                }]
            }
        });

        let request = BattleRequest::parse(&json).unwrap();
        let mut battle = TrackedBattle::new();

        battle.apply_request(&request);

        assert_eq!(battle.knowledge(), BattleKnowledge::Player(Player::P1));
        assert_eq!(battle.viewpoint(), Some(Player::P1));

        let me = battle.me().unwrap();
        assert_eq!(me.username, "Alice");
        assert_eq!(me.pokemon.len(), 1);
        assert_eq!(me.pokemon[0].known_ability.as_deref(), Some("Static"));
    }

    #[test]
    fn test_apply_replay_log_in_omniscient_mode() {
        let log = r#"|inactive|Battle timer is ON: inactive players will automatically lose when time's up.
|J|Pokebasket
|J|Alf
|player|p1|Pokebasket|278
|player|p2|Alf|44
|gametype|singles
|gen|3
|tier|[Gen 3] OU
|rule|Sleep Clause Mod: Limit one foe put to sleep
|rule|Species Clause: Limit one of each Pokémon
|rule|OHKO Clause: OHKO moves are banned
|rule|Moody Clause: Moody is banned
|rule|Evasion Moves Clause: Evasion moves are banned
|rule|Endless Battle Clause: Forcing endless battles is banned
|rule|HP Percentage Mod: HP is shown in percentages
|
|start
|switch|p1a: Hill|Salamence, M|331/331
|switch|p2a: Salamence|Salamence, M|331/331
|-ability|p1a: Hill|Intimidate|[of] p2a: Salamence
|-unboost|p2a: Salamence|atk|1
|-ability|p2a: Salamence|Intimidate|[of] p1a: Hill
|-unboost|p1a: Hill|atk|1
|turn|1
|J|Da Raikage
|J|IZANAGI-N0-0KAMI
|J|Tesung
|J|Malekith
|
|switch|p1a: Lutra|Milotic, F|394/394
|move|p2a: Salamence|Dragon Claw|p1a: Lutra
|-damage|p1a: Lutra|267/394
|
|-heal|p1a: Lutra|291/394|[from] item: Leftovers
|turn|2
|
|switch|p2a: Snorlax|Snorlax, M|497/497
|move|p1a: Lutra|Recover|p1a: Lutra
|-heal|p1a: Lutra|394/394
|
|turn|3
|
|switch|p1a: Conflict|Skarmory, F|334/334
|move|p2a: Snorlax|Body Slam|p1a: Conflict
|-resisted|p1a: Conflict
|-damage|p1a: Conflict|283/334
|
|-heal|p1a: Conflict|303/334|[from] item: Leftovers
|turn|4
|
|switch|p2a: Salamence|Salamence, M|331/331
|-ability|p2a: Salamence|Intimidate|[of] p1a: Conflict
|-unboost|p1a: Conflict|atk|1
|move|p1a: Conflict|Spikes|p2a: Salamence
|-sidestart|p2: Alf|Spikes
|
|-heal|p1a: Conflict|323/334|[from] item: Leftovers
|turn|5
|J|Sken
|
|switch|p1a: Lutra|Milotic, F|394/394
|move|p2a: Salamence|Dragon Claw|p1a: Lutra
|-crit|p1a: Lutra
|-damage|p1a: Lutra|151/394
|
|-heal|p1a: Lutra|175/394|[from] item: Leftovers
|turn|6
|
|switch|p2a: Snorlax|Snorlax, M|497/497
|-damage|p2a: Snorlax|435/497|[from] Spikes
|move|p1a: Lutra|Recover|p1a: Lutra
|-heal|p1a: Lutra|372/394
|
|-heal|p1a: Lutra|394/394|[from] item: Leftovers
|-heal|p2a: Snorlax|466/497|[from] item: Leftovers
|turn|7
|
|switch|p1a: Conflict|Skarmory, F|323/334
|move|p2a: Snorlax|Curse|p2a: Snorlax
|-boost|p2a: Snorlax|atk|1
|-boost|p2a: Snorlax|def|1
|-unboost|p2a: Snorlax|spe|1
|
|-heal|p1a: Conflict|334/334|[from] item: Leftovers
|-heal|p2a: Snorlax|497/497|[from] item: Leftovers
|turn|8
|J|Jirachee
|
|move|p1a: Conflict|Spikes|p2a: Snorlax
|-sidestart|p2: Alf|Spikes
|move|p2a: Snorlax|Self-Destruct|p1a: Conflict
|-resisted|p1a: Conflict
|-damage|p1a: Conflict|28/334
|faint|p2a: Snorlax
|L|Jirachee
|
|switch|p2a: Swampert|Swampert, M|341/341
|-damage|p2a: Swampert|285/341|[from] Spikes
|
|-heal|p1a: Conflict|48/334|[from] item: Leftovers
|turn|9
|J|avaawa
|
|move|p1a: Conflict|Spikes|p2a: Swampert
|-sidestart|p2: Alf|Spikes
|move|p2a: Swampert|Ice Beam|p1a: Conflict
|-damage|p1a: Conflict|0 fnt
|faint|p1a: Conflict
|
|switch|p1a: Lutra|Milotic, F|394/394
|
|turn|10
|
|switch|p2a: Salamence|Salamence, M|331/331
|-ability|p2a: Salamence|Intimidate|[of] p1a: Lutra
|-unboost|p1a: Lutra|atk|1
|move|p1a: Lutra|Surf|p2a: Salamence
|-resisted|p2a: Salamence
|-damage|p2a: Salamence|253/331
|
|turn|11
|
|switch|p2a: Metagross|Metagross|347/347
|-damage|p2a: Metagross|261/347|[from] Spikes
|move|p1a: Lutra|Ice Beam|p2a: Metagross
|-resisted|p2a: Metagross
|-damage|p2a: Metagross|220/347
|
|-heal|p2a: Metagross|241/347|[from] item: Leftovers
|turn|12
|
|switch|p1a: Hill|Salamence, M|331/331
|-ability|p1a: Hill|Intimidate|[of] p2a: Metagross
|-fail|p2a: Metagross|unboost|[from] ability: Clear Body|[of] p2a: Metagross
|move|p2a: Metagross|Psychic|p1a: Hill
|-damage|p1a: Hill|163/331
|
|-heal|p1a: Hill|183/331|[from] item: Leftovers
|-heal|p2a: Metagross|262/347|[from] item: Leftovers
|turn|13
|
|switch|p2a: Salamence|Salamence, M|253/331
|-ability|p2a: Salamence|Intimidate|[of] p1a: Hill
|-unboost|p1a: Hill|atk|1
|move|p1a: Hill|Fire Blast|p2a: Salamence
|-resisted|p2a: Salamence
|-damage|p2a: Salamence|158/331
|
|-heal|p1a: Hill|203/331|[from] item: Leftovers
|turn|14
|
|switch|p1a: Lutra|Milotic, F|394/394
|move|p2a: Salamence|Dragon Claw|p1a: Lutra
|-damage|p1a: Lutra|280/394
|
|-heal|p1a: Lutra|304/394|[from] item: Leftovers
|turn|15
|
|move|p2a: Salamence|Hidden Power|p1a: Lutra
|-supereffective|p1a: Lutra
|-damage|p1a: Lutra|182/394
|move|p1a: Lutra|Recover|p1a: Lutra
|-heal|p1a: Lutra|379/394
|
|-heal|p1a: Lutra|394/394|[from] item: Leftovers
|turn|16
|
|switch|p2a: Tyranitar|Tyranitar, M|345/345
|-damage|p2a: Tyranitar|259/345|[from] Spikes
|-weather|Sandstorm|[from] ability: Sand Stream|[of] p2a: Tyranitar
|move|p1a: Lutra|Surf|p2a: Tyranitar
|-supereffective|p2a: Tyranitar
|-damage|p2a: Tyranitar|47/345
|
|-weather|Sandstorm|[upkeep]
|-damage|p1a: Lutra|370/394|[from] sandstorm
|-heal|p2a: Tyranitar|68/345|[from] item: Leftovers
|-heal|p1a: Lutra|394/394|[from] item: Leftovers
|turn|17
|
|move|p2a: Tyranitar|Rock Slide|p1a: Lutra
|-damage|p1a: Lutra|262/394
|move|p1a: Lutra|Surf|p2a: Tyranitar
|-supereffective|p2a: Tyranitar
|-damage|p2a: Tyranitar|0 fnt
|faint|p2a: Tyranitar
|
|switch|p2a: Metagross|Metagross|262/347
|-damage|p2a: Metagross|176/347|[from] Spikes
|
|-weather|Sandstorm|[upkeep]
|-damage|p1a: Lutra|238/394|[from] sandstorm
|-heal|p1a: Lutra|262/394|[from] item: Leftovers
|-heal|p2a: Metagross|197/347|[from] item: Leftovers
|turn|18
|
|switch|p1a: Reik|Raikou|322/322
|move|p2a: Metagross|Explosion|p1a: Reik
|-damage|p1a: Reik|0 fnt
|faint|p2a: Metagross
|faint|p1a: Reik
|
|switch|p2a: Aerodactyl|Aerodactyl, F|300/300
|switch|p1a: Lutra|Milotic, F|262/394
|
|-weather|Sandstorm|[upkeep]
|-damage|p1a: Lutra|238/394|[from] sandstorm
|-heal|p1a: Lutra|262/394|[from] item: Leftovers
|turn|19
|
|move|p2a: Aerodactyl|Rock Slide|p1a: Lutra
|-miss|p2a: Aerodactyl|p1a: Lutra
|move|p1a: Lutra|Recover|p1a: Lutra
|-heal|p1a: Lutra|394/394
|
|-weather|Sandstorm|[upkeep]
|-damage|p1a: Lutra|370/394|[from] sandstorm
|-heal|p1a: Lutra|394/394|[from] item: Leftovers
|turn|20
|
|move|p2a: Aerodactyl|Rock Slide|p1a: Lutra
|-damage|p1a: Lutra|240/394
|cant|p1a: Lutra|flinch
|
|-weather|Sandstorm|[upkeep]
|-damage|p1a: Lutra|216/394|[from] sandstorm
|-heal|p1a: Lutra|240/394|[from] item: Leftovers
|turn|21
|
|move|p2a: Aerodactyl|Rock Slide|p1a: Lutra
|-damage|p1a: Lutra|96/394
|cant|p1a: Lutra|flinch
|
|-weather|Sandstorm|[upkeep]
|-damage|p1a: Lutra|72/394|[from] sandstorm
|-heal|p1a: Lutra|96/394|[from] item: Leftovers
|turn|22
|c|★Pokebasket|ah
|c|★Pokebasket|xD
|J|Cat B1ack
|
|switch|p1a: Hill|Salamence, M|203/331
|-ability|p1a: Hill|Intimidate|[of] p2a: Aerodactyl
|-unboost|p2a: Aerodactyl|atk|1
|move|p2a: Aerodactyl|Rock Slide|p1a: Hill
|-supereffective|p1a: Hill
|-damage|p1a: Hill|0 fnt
|faint|p1a: Hill
|
|switch|p1a: PROBLEMS|Tyranitar, M|345/345
|
|-weather|Sandstorm|[upkeep]
|turn|23
|
|switch|p2a: Salamence|Salamence, M|158/331
|-ability|p2a: Salamence|Intimidate|[of] p1a: PROBLEMS
|-unboost|p1a: PROBLEMS|atk|1
|move|p1a: PROBLEMS|Dragon Dance|p1a: PROBLEMS
|-boost|p1a: PROBLEMS|atk|1
|-boost|p1a: PROBLEMS|spe|1
|
|-weather|Sandstorm|[upkeep]
|-damage|p2a: Salamence|138/331|[from] sandstorm
|turn|24
|
|move|p1a: PROBLEMS|Rock Slide|p2a: Salamence
|-supereffective|p2a: Salamence
|-damage|p2a: Salamence|0 fnt
|faint|p2a: Salamence
|
|switch|p2a: Swampert|Swampert, M|285/341
|-damage|p2a: Swampert|200/341|[from] Spikes
|
|-weather|Sandstorm|[upkeep]
|turn|25
|
|move|p1a: PROBLEMS|Dragon Dance|p1a: PROBLEMS
|-boost|p1a: PROBLEMS|atk|1
|-boost|p1a: PROBLEMS|spe|1
|move|p2a: Swampert|Surf|p1a: PROBLEMS
|-supereffective|p1a: PROBLEMS
|-damage|p1a: PROBLEMS|79/345
|
|-weather|Sandstorm|[upkeep]
|-heal|p1a: PROBLEMS|100/345|[from] item: Leftovers
|turn|26
|
|move|p1a: PROBLEMS|Earthquake|p2a: Swampert
|-damage|p2a: Swampert|0 fnt
|faint|p2a: Swampert
|
|switch|p2a: Aerodactyl|Aerodactyl, F|300/300
|
|-weather|Sandstorm|[upkeep]
|-heal|p1a: PROBLEMS|121/345|[from] item: Leftovers
|turn|27
|
|move|p1a: PROBLEMS|Rock Slide|p2a: Aerodactyl
|-supereffective|p2a: Aerodactyl
|-damage|p2a: Aerodactyl|0 fnt
|faint|p2a: Aerodactyl
|
|win|Pokebasket"#;

        let mut battle = TrackedBattle::omniscient();
        battle.set_viewpoint(Player::P1);

        for line in log.lines() {
            let message = parse_server_message(line).unwrap();
            battle.apply_message(&message);
        }

        assert_eq!(battle.knowledge(), BattleKnowledge::Omniscient);
        assert_eq!(battle.turn, 27);
        assert_eq!(battle.winner.as_deref(), Some("Pokebasket"));
        assert!(battle.ended);
        assert_eq!(battle.field.weather, Some(Weather::Sand));

        let p1 = battle.get_side(Player::P1).unwrap();
        let p2 = battle.get_side(Player::P2).unwrap();

        assert_eq!(p1.username, "Pokebasket");
        assert_eq!(p2.username, "Alf");
        assert_eq!(p2.condition_layers(SideCondition::Spikes), 3);
        assert!(p2.all_fainted());

        let active = p1.active_pokemon().unwrap();
        assert_eq!(active.identity.species, "Tyranitar");
        assert_eq!(active.hp_current, 121);
        assert_eq!(active.hp_max, Some(345));

        let milotic = p1.find_pokemon("Lutra").unwrap();
        assert_eq!(p1.pokemon[milotic].hp_current, 96);
        assert_eq!(p1.pokemon[milotic].hp_max, Some(394));
    }
}
