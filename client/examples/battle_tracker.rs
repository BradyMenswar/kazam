//! Battle State Tracker Example
//!
//! This bot joins unrated random battles and uses kazam-battle's TrackedBattle
//! to accumulate and print battle state at the end of each turn.

use anyhow::Result;
use kazam_battle::TrackedBattle;
use kazam_client::{
    BattleRequest, KazamClient, KazamHandle, KazamHandler, RoomType, SHOWDOWN_URL, ServerMessage,
    User,
};
use rand::seq::SliceRandom;
use std::collections::HashMap;

struct BattleTrackerBot {
    handle: KazamHandle,
    /// Track battle state per room
    battles: HashMap<String, TrackedBattle>,
}

impl BattleTrackerBot {
    fn new(handle: KazamHandle) -> Self {
        Self {
            handle,
            battles: HashMap::new(),
        }
    }

    fn get_or_create_battle(&mut self, room_id: &str) -> &mut TrackedBattle {
        self.battles
            .entry(room_id.to_string())
            .or_insert_with(TrackedBattle::new)
    }

    fn make_choice(&self, room_id: &str, request: &BattleRequest) {
        let rqid = request.rqid;

        if request.wait {
            return;
        }

        // Handle team preview
        if request.team_preview {
            let team_size = request.side.as_ref().map(|s| s.pokemon.len()).unwrap_or(6);
            let order: String = (1..=team_size).map(|i| i.to_string()).collect();
            self.handle
                .choose(room_id, &format!("team {}", order), rqid)
                .ok();
            return;
        }

        // Handle force switch
        if request.is_force_switch() {
            if let Some(choice) = self.pick_switch(request) {
                self.handle.choose(room_id, &choice, rqid).ok();
                return;
            }
        }

        // Normal turn - pick a random move
        if let Some(choice) = self.pick_action(request) {
            self.handle.choose(room_id, &choice, rqid).ok();
        }
    }

    fn pick_action(&self, request: &BattleRequest) -> Option<String> {
        let mut rng = rand::thread_rng();
        let mut choices = Vec::new();

        if let Some(active) = request.active.as_ref().and_then(|a| a.first()) {
            for (i, _move) in active.available_moves() {
                choices.push(format!("move {}", i + 1));
            }
        }

        choices.choose(&mut rng).cloned()
    }

    fn pick_switch(&self, request: &BattleRequest) -> Option<String> {
        let mut rng = rand::thread_rng();

        if let Some(side) = &request.side {
            let switches: Vec<String> = side
                .pokemon
                .iter()
                .enumerate()
                .filter(|(_, p)| !p.active && !p.is_fainted())
                .map(|(i, _)| format!("switch {}", i + 1))
                .collect();

            return switches.choose(&mut rng).cloned();
        }

        None
    }

    fn print_battle_state(&self, room_id: &str) {
        let Some(battle) = self.battles.get(room_id) else {
            return;
        };

        println!("\n{}", "=".repeat(60));
        println!("BATTLE STATE - Turn {}", battle.turn);
        println!("{}", "=".repeat(60));

        // Print metadata
        if let Some(game_type) = &battle.game_type {
            println!(
                "Format: Gen {} {:?} - {}",
                battle.generation, game_type, battle.tier
            );
        }

        // Print field conditions
        let field = &battle.field;
        let mut field_effects = Vec::new();
        if let Some(weather) = &field.weather {
            field_effects.push(format!("Weather: {:?}", weather));
        }
        if let Some(terrain) = &field.terrain {
            field_effects.push(format!("Terrain: {:?}", terrain));
        }
        if field.trick_room {
            field_effects.push("Trick Room".to_string());
        }
        if field.gravity {
            field_effects.push("Gravity".to_string());
        }
        if !field_effects.is_empty() {
            println!("Field: {}", field_effects.join(", "));
        }

        println!("{}", "-".repeat(60));

        // Print each side
        for side in battle.sides() {
            let is_me = battle
                .perspective()
                .map(|p| p == side.player)
                .unwrap_or(false);
            let label = if is_me { "(You)" } else { "(Opponent)" };

            println!(
                "\n{} {} {}",
                side.player.as_str().to_uppercase(),
                side.username,
                label
            );

            // Print side conditions
            if !side.conditions.is_empty() {
                let conditions: Vec<String> = side
                    .conditions
                    .iter()
                    .map(|(c, state)| {
                        if state.layers > 1 {
                            format!("{:?} x{}", c, state.layers)
                        } else {
                            format!("{:?}", c)
                        }
                    })
                    .collect();
                println!("  Conditions: {}", conditions.join(", "));
            }

            // Print active Pokemon
            for active in side.get_active() {
                println!("  Active: {}", format_pokemon(active, true));
            }

            // Print bench
            let bench: Vec<_> = side.get_bench().collect();
            if !bench.is_empty() {
                println!("  Bench:");
                for (_idx, poke) in bench {
                    println!("    - {}", format_pokemon(poke, false));
                }
            }
        }

        println!("{}", "=".repeat(60));
    }
}

fn format_pokemon(poke: &kazam_battle::PokemonState, show_details: bool) -> String {
    let mut parts = Vec::new();

    // Name/species
    parts.push(poke.name().to_string());

    // Level if not 100
    if poke.identity.level != 100 {
        parts.push(format!("L{}", poke.identity.level));
    }

    // HP
    if let Some(max) = poke.hp_max {
        parts.push(format!("{}/{}HP", poke.hp_current, max));
    } else if poke.hp_current > 0 {
        parts.push(format!("{}%", poke.hp_current));
    }

    // Status
    if poke.fainted {
        parts.push("(fainted)".to_string());
    } else if let Some(status) = &poke.status {
        parts.push(format!("[{:?}]", status));
    }

    if show_details {
        // Boosts
        let boosts = &poke.boosts;
        let mut boost_parts = Vec::new();
        if boosts.atk != 0 {
            boost_parts.push(format!("Atk{:+}", boosts.atk));
        }
        if boosts.def != 0 {
            boost_parts.push(format!("Def{:+}", boosts.def));
        }
        if boosts.spa != 0 {
            boost_parts.push(format!("SpA{:+}", boosts.spa));
        }
        if boosts.spd != 0 {
            boost_parts.push(format!("SpD{:+}", boosts.spd));
        }
        if boosts.spe != 0 {
            boost_parts.push(format!("Spe{:+}", boosts.spe));
        }
        if !boost_parts.is_empty() {
            parts.push(format!("({})", boost_parts.join(" ")));
        }

        // Volatiles (show up to 3)
        if !poke.volatiles.is_empty() {
            let vol_strs: Vec<_> = poke
                .volatiles
                .iter()
                .take(3)
                .map(|v| format!("{:?}", v))
                .collect();
            let more = if poke.volatiles.len() > 3 {
                format!(" +{}", poke.volatiles.len() - 3)
            } else {
                String::new()
            };
            parts.push(format!("[{}{}]", vol_strs.join(", "), more));
        }

        // Known info
        if let Some(ability) = &poke.known_ability {
            parts.push(format!("Ability:{}", ability));
        }
        if let Some(item) = &poke.known_item {
            if !poke.item_consumed {
                parts.push(format!("Item:{}", item));
            }
        }
    }

    parts.join(" ")
}

impl KazamHandler for BattleTrackerBot {
    async fn on_challstr(&mut self, challstr: &str) {
        println!("Logging in...");
        self.handle
            .login("bmax117", "dragon117", challstr)
            .await
            .expect("Failed to login");
    }

    async fn on_logged_in(&mut self, user: &User) {
        println!("Logged in as: {}{}", user.rank, user.username);
        println!("Searching for a random battle...");
        self.handle
            .search("gen9randombattle")
            .expect("Failed to search");
    }

    async fn on_init(&mut self, room_id: &str, room_type: &RoomType) {
        if *room_type == RoomType::Battle {
            println!("Joined battle: {}", room_id);
            // Create a new battle tracker for this room
            self.battles
                .insert(room_id.to_string(), TrackedBattle::new());
        }
    }

    async fn on_request(&mut self, room_id: &str, request: &BattleRequest) {
        // Update battle state from the request (gives us full team info)
        let battle = self.get_or_create_battle(room_id);
        battle.update_from_request(request);

        // Make our move
        self.make_choice(room_id, request);
    }

    async fn on_turn(&mut self, room_id: &str, _turn: u32) {
        // Print accumulated state at the start of each turn
        self.print_battle_state(room_id);
    }

    async fn on_battle_message(&mut self, room_id: Option<&str>, message: ServerMessage) {
        // Feed all battle messages to TrackedBattle
        if let Some(rid) = room_id {
            let battle = self.get_or_create_battle(rid);
            battle.update(&message);
        }
    }

    async fn on_win(&mut self, room_id: &str, winner: &str) {
        // Print final state
        self.print_battle_state(room_id);
        println!("\n{} won the battle!", winner);

        // Clean up
        self.battles.remove(room_id);

        // Search for another battle
        println!("\nSearching for another battle...");
        self.handle.search("gen9randombattle").ok();
    }

    async fn on_tie(&mut self, room_id: &str) {
        self.print_battle_state(room_id);
        println!("\nThe battle ended in a tie!");
        self.battles.remove(room_id);
        println!("\nSearching for another battle...");
        self.handle.search("gen9randombattle").ok();
    }

    async fn on_popup(&mut self, message: &str) {
        println!("Popup: {}", message.replace("||", "\n"));
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("Battle State Tracker");
    println!("====================");
    println!("This bot tracks battle state using kazam-battle::TrackedBattle");
    println!("and prints the accumulated state at the start of each turn.\n");
    println!("Connecting to Pokemon Showdown...");

    let mut client = KazamClient::connect(SHOWDOWN_URL).await?;
    println!("Connected!");

    let mut handler = BattleTrackerBot::new(client.handle());

    client.run(&mut handler).await
}
