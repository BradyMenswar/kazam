# kazam-battle

Battle state tracking crate for Pokemon Showdown clients. Builds on top of `kazam-client` to provide comprehensive battle state management.

## Goals

- Track complete battle state from server messages
- Provide query API for bot decision-making
- Zero-copy where possible, minimal allocations
- Optional - clients can use raw handlers if they don't need state tracking

## Core Types

### Battle

The main entry point. Maintains full state for a single battle.

```rust
pub struct Battle {
    pub info: BattleInfo,           // From initialization
    pub turn: u32,
    pub weather: Option<Weather>,
    pub terrain: Option<Terrain>,
    pub players: [PlayerState; 2],  // p1, p2
    pub ended: bool,
    pub winner: Option<Player>,
}

impl Battle {
    pub fn new() -> Self;
    pub fn update(&mut self, message: &ServerMessage);

    // Query API
    pub fn me(&self) -> &PlayerState;      // Requires set_perspective()
    pub fn opponent(&self) -> &PlayerState;
    pub fn player(&self, p: Player) -> &PlayerState;
    pub fn active(&self, p: Player) -> Option<&Pokemon>;
    pub fn is_my_turn(&self) -> bool;
}
```

### PlayerState

State for one side of the battle.

```rust
pub struct PlayerState {
    pub player: Player,
    pub username: String,
    pub team: Vec<Pokemon>,
    pub active: Vec<usize>,         // Indices into team (1 for singles, 2 for doubles)
    pub side_conditions: HashSet<SideCondition>,
}

impl PlayerState {
    pub fn active_pokemon(&self) -> impl Iterator<Item = &Pokemon>;
    pub fn bench(&self) -> impl Iterator<Item = &Pokemon>;
    pub fn alive_count(&self) -> usize;
    pub fn has_condition(&self, cond: SideCondition) -> bool;
}
```

### Pokemon (Battle State)

Extended Pokemon state beyond protocol's Pokemon identifier.

```rust
pub struct Pokemon {
    // Identity
    pub species: String,
    pub nickname: Option<String>,
    pub level: u8,
    pub gender: Option<Gender>,

    // Current state
    pub hp: u32,
    pub max_hp: u32,
    pub status: Option<Status>,
    pub fainted: bool,

    // Stat stages (only tracked for active)
    pub boosts: StatBoosts,

    // Known info (revealed during battle)
    pub known_moves: Vec<String>,
    pub known_ability: Option<String>,
    pub known_item: Option<String>,
    pub item_consumed: bool,
    pub terastallized: Option<String>,
}

impl Pokemon {
    pub fn hp_percent(&self) -> u32;
    pub fn is_fainted(&self) -> bool;
    pub fn is_active(&self) -> bool;
}
```

### StatBoosts

```rust
pub struct StatBoosts {
    pub atk: i8,    // -6 to +6
    pub def: i8,
    pub spa: i8,
    pub spd: i8,
    pub spe: i8,
    pub accuracy: i8,
    pub evasion: i8,
}
```

### Field Conditions

```rust
pub enum Weather {
    Sun, HarshSun,
    Rain, HeavyRain,
    Sand, Hail, Snow,
}

pub enum Terrain {
    Electric, Grassy, Misty, Psychic,
}

pub enum SideCondition {
    Reflect, LightScreen, AuroraVeil,
    Spikes(u8),      // 1-3 layers
    ToxicSpikes(u8), // 1-2 layers
    StealthRock,
    StickyWeb,
    Tailwind,
    // etc.
}
```

## Integration with kazam-client

```rust
use kazam_client::{KazamClient, KazamHandler, ServerMessage};
use kazam_battle::Battle;
use std::collections::HashMap;

struct MyBot {
    handle: KazamHandle,
    battles: HashMap<String, Battle>,
    username: String,
}

impl KazamHandler for MyBot {
    async fn on_logged_in(&mut self, user: &User) {
        self.username = user.username.clone();
    }

    async fn on_init(&mut self, room_id: &str, room_type: &RoomType) {
        if *room_type == RoomType::Battle {
            let mut battle = Battle::new();
            battle.set_perspective(&self.username);
            self.battles.insert(room_id.to_string(), battle);
        }
    }

    async fn on_battle_message(&mut self, room_id: Option<&str>, message: ServerMessage) {
        if let Some(room_id) = room_id {
            if let Some(battle) = self.battles.get_mut(room_id) {
                battle.update(&message);
            }
        }
    }

    async fn on_request(&mut self, room_id: &str, request: &BattleRequest) {
        let battle = self.battles.get(room_id).unwrap();

        // Now you can make decisions with full state
        let my_pokemon = battle.me().active_pokemon().next().unwrap();
        let their_pokemon = battle.opponent().active_pokemon().next().unwrap();

        println!("My {} ({}%) vs their {} ({}%)",
            my_pokemon.species, my_pokemon.hp_percent(),
            their_pokemon.species, their_pokemon.hp_percent());
    }

    async fn on_win(&mut self, room_id: &str, _winner: &str) {
        self.battles.remove(room_id);
    }
}
```

## State Update Logic

The `Battle::update()` method handles all message types:

| Message | State Update |
|---------|--------------|
| `BattlePlayer` | Set player info |
| `Switch/Drag` | Update active Pokemon, reset boosts |
| `Damage/Heal/SetHp` | Update Pokemon HP |
| `Faint` | Mark Pokemon fainted |
| `Boost/Unboost` | Update stat stages |
| `Status/CureStatus` | Update status condition |
| `Weather` | Set field weather |
| `FieldStart/FieldEnd` | Set terrain |
| `SideStart/SideEnd` | Add/remove side conditions |
| `Item/EndItem` | Track revealed/consumed items |
| `Ability` | Track revealed abilities |
| `Move` | Track revealed moves |
| `Turn` | Increment turn counter |
| `Win/Tie` | Mark battle ended |

## Future Additions

### Damage Calculator

```rust
pub fn calc_damage(
    attacker: &Pokemon,
    defender: &Pokemon,
    move: &Move,
    field: &FieldState,
) -> DamageRange;
```

### Type Chart

```rust
pub fn type_effectiveness(attacking: Type, defending: &[Type]) -> f32;
```

### Move Database

Optional feature flag to include move data (power, type, category, effects).

### Speed Tier Calculation

```rust
pub fn speed_order(battle: &Battle) -> Vec<(Player, usize)>;
```

## Crate Structure

```
kazam-battle/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── battle.rs       # Battle struct and update logic
    ├── player.rs       # PlayerState
    ├── pokemon.rs      # Pokemon state
    ├── field.rs        # Weather, terrain, side conditions
    ├── boosts.rs       # Stat boost tracking
    └── query.rs        # Query helpers
```

## Non-Goals (for v1)

- Prediction/AI logic
- Team building/validation
- Replay parsing (different from live battles)
- Multi-battle management (that's the client's job)
