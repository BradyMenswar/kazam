//! Random Battle Bot Example
//!
//! This bot joins unrated random battles and makes random moves.
//! It demonstrates how to use the typed battle handlers.

use anyhow::Result;
use kazam_client::{
    BattleRequest, HpStatus, KazamClient, KazamHandle, KazamHandler, Pokemon, PokemonDetails,
    RoomType, SHOWDOWN_URL, User,
};
use rand::seq::SliceRandom;

struct RandomBattleBot {
    handle: KazamHandle,
}

impl RandomBattleBot {
    fn make_choice(&self, room_id: &str, request: &BattleRequest) {
        let rqid = request.rqid;

        // Check if we need to wait
        if request.wait {
            println!("[{}] Waiting for opponent...", room_id);
            return;
        }

        // Handle team preview
        if request.team_preview {
            let team_size = request.side.as_ref().map(|s| s.pokemon.len()).unwrap_or(6);
            let order: String = (1..=team_size).map(|i| i.to_string()).collect();
            println!("[{}] Team preview: team {}", room_id, order);
            self.handle
                .choose(room_id, &format!("team {}", order), rqid)
                .ok();
            return;
        }

        // Handle force switch
        if request.is_force_switch() {
            if let Some(choice) = self.pick_switch(request) {
                println!("[{}] Force switch: {}", room_id, choice);
                self.handle.choose(room_id, &choice, rqid).ok();
                return;
            }
        }

        // Normal turn - pick a random move or switch
        if let Some(choice) = self.pick_action(request) {
            println!("[{}] Choosing: {}", room_id, choice);
            self.handle.choose(room_id, &choice, rqid).ok();
        }
    }

    fn pick_action(&self, request: &BattleRequest) -> Option<String> {
        let mut rng = rand::thread_rng();
        let mut choices = Vec::new();

        // Get available moves from active pokemon (no voluntary switches for faster testing)
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
}

impl KazamHandler for RandomBattleBot {
    async fn on_challstr(&mut self, challstr: &str) {
        println!("Logging in...");
        // NOTE: Replace with your own credentials
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

    async fn on_name_taken(&mut self, username: &str, message: &str) {
        println!("Login failed for {}: {}", username, message);
    }

    async fn on_init(&mut self, room_id: &str, room_type: &RoomType) {
        if *room_type == RoomType::Battle {
            println!("Joined battle: {}", room_id);
        }
    }

    // ===================
    // Typed Battle Handlers
    // ===================

    async fn on_request(&mut self, room_id: &str, request: &BattleRequest) {
        self.make_choice(room_id, request);
    }

    async fn on_turn(&mut self, room_id: &str, turn: u32) {
        println!("[{}] === Turn {} ===", room_id, turn);
    }

    async fn on_switch(
        &mut self,
        room_id: &str,
        pokemon: &Pokemon,
        details: &PokemonDetails,
        _hp_status: Option<&HpStatus>,
        is_drag: bool,
    ) {
        let action = if is_drag {
            "was dragged out"
        } else {
            "sent out"
        };
        println!(
            "[{}] {} {} {}!",
            room_id,
            pokemon.player.as_str(),
            action,
            details.species
        );
    }

    async fn on_move_used(
        &mut self,
        room_id: &str,
        pokemon: &Pokemon,
        move_name: &str,
        _target: Option<&Pokemon>,
    ) {
        println!("[{}] {} used {}!", room_id, pokemon.name, move_name);
    }

    async fn on_faint(&mut self, room_id: &str, pokemon: &Pokemon) {
        println!("[{}] {} fainted!", room_id, pokemon.name);
    }

    async fn on_win(&mut self, room_id: &str, winner: &str) {
        println!("[{}] {} won the battle!", room_id, winner);
        println!("Searching for another battle...");
        self.handle.search("gen9randombattle").ok();
    }

    async fn on_tie(&mut self, room_id: &str) {
        println!("[{}] The battle ended in a tie!", room_id);
        println!("Searching for another battle...");
        self.handle.search("gen9randombattle").ok();
    }

    async fn on_damage(&mut self, room_id: &str, pokemon: &Pokemon, hp_status: Option<&HpStatus>) {
        if let Some(hp) = hp_status {
            if hp.max.is_some() {
                println!(
                    "[{}] {} took damage: {}/{}",
                    room_id,
                    pokemon.name,
                    hp.current,
                    hp.max.unwrap()
                );
            }
        }
    }

    async fn on_super_effective(&mut self, room_id: &str, _pokemon: &Pokemon) {
        println!("[{}] It's super effective!", room_id);
    }

    async fn on_crit(&mut self, room_id: &str, _pokemon: &Pokemon) {
        println!("[{}] A critical hit!", room_id);
    }

    async fn on_popup(&mut self, message: &str) {
        println!("Popup: {}", message.replace("||", "\n"));
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("Random Battle Bot");
    println!("==================");
    println!("Connecting to Pokemon Showdown...");

    let mut client = KazamClient::connect(SHOWDOWN_URL).await?;
    println!("Connected!");

    let mut handler = RandomBattleBot {
        handle: client.handle(),
    };

    client.run(&mut handler).await
}
