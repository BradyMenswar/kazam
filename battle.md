# kazam-battle

Battle state tracking and domain types for Pokemon Showdown. This crate provides the shared type system used by both state tracking (for bots) and simulation (for prediction/training).

## Architecture Overview

```ignore
┌─────────────────────────────────────────────────────────────────────────────┐
│                           Crate Relationships                                │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│                            kazam-protocol                                    │
│                          ┌───────────────────┐                               │
│                          │ Wire Format:      │                               │
│                          │ - ServerMessage   │                               │
│                          │ - ClientCommand   │                               │
│                          │ - BattleRequest   │                               │
│                          │ - Pokemon (ident) │                               │
│                          │ - HpStatus        │                               │
│                          └─────────┬─────────┘                               │
│                                    │                                         │
│                                    ▼                                         │
│                             kazam-battle                                     │
│                          ┌───────────────────┐                               │
│                          │ Domain Types:     │                               │
│                          │ - Type, TypeChart │                               │
│                          │ - Status, Volatile│                               │
│                          │ - StatStages      │                               │
│                          │ - Weather,Terrain │                               │
│                          │ - SideCondition   │                               │
│                          │ - PokemonState    │                               │
│                          │ - SideState       │                               │
│                          │ - FieldState      │                               │
│                          │                   │                               │
│                          │ State Tracking:   │                               │
│                          │ - TrackedBattle   │                               │
│                          │ - BattleUpdater   │                               │
│                          └─────────┬─────────┘                               │
│                                    │                                         │
│                    ┌───────────────┴───────────────┐                         │
│                    │                               │                         │
│                    ▼                               ▼                         │
│             kazam-client                   kazam-simulator                   │
│          ┌───────────────────┐          ┌───────────────────┐               │
│          │ WebSocket I/O     │          │ Full Simulation   │               │
│          │ KazamHandler      │          │ Event System      │               │
│          │ Uses TrackedBattle│          │ Damage Calc       │               │
│          │ for bot state     │          │ Uses domain types │               │
│          └───────────────────┘          └───────────────────┘               │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Design Rationale

### Why separate `kazam-battle` from `kazam-simulator`?

1. **Most bots only need state tracking**
   - Tracking battle state from server messages is sufficient for many bots
   - Full simulation requires loading all game data (~10MB+)
   - Simulation adds significant compile-time and runtime overhead

2. **Separation of concerns**
   - **Tracking**: Reconstructs state from incomplete information (what we observe)
   - **Simulation**: Maintains complete authoritative state (what actually is)

3. **Shared domain types**
   - Both tracking and simulation need the same concepts (Pokemon, stats, conditions)
   - Defining types once in `kazam-battle` prevents duplication
   - Simulator can extend these types with simulation-specific fields

### Dependency Direction

```
kazam-protocol  (wire format, no game logic)
       │
       ▼
kazam-battle    (domain types + tracking, lightweight)
       │
       ▼
kazam-simulator (full mechanics, heavy - OPTIONAL)
```

**Key principle**: `kazam-battle` does NOT depend on `kazam-simulator`. Users who only need state tracking don't pay for simulation.

---

## Crate Structure

```
kazam-battle/
├── Cargo.toml
└── src/
    ├── lib.rs              # Public exports
    │
    ├── types/              # Shared domain types
    │   ├── mod.rs
    │   ├── pokemon_type.rs # Type enum, effectiveness chart
    │   ├── status.rs       # Status, Volatile enums
    │   ├── stats.rs        # StatStages, Stat helpers
    │   ├── conditions.rs   # Weather, Terrain, SideCondition
    │   ├── pokemon.rs      # PokemonState, PokemonIdentity
    │   ├── side.rs         # SideState, SideConditionState
    │   └── field.rs        # FieldState, PseudoWeather
    │
    ├── tracking/           # State reconstruction from messages
    │   ├── mod.rs
    │   ├── battle.rs       # TrackedBattle
    │   └── updater.rs      # Message -> state updates
    │
    └── query/              # Query helpers for decision making
        ├── mod.rs
        ├── matchup.rs      # Type matchup helpers
        └── eval.rs         # Position evaluation helpers
```

---

## Domain Types

### Type System

```rust
/// Pokemon types (18 types as of Gen 6+)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Type {
    Normal = 0, Fire, Water, Electric, Grass, Ice,
    Fighting, Poison, Ground, Flying, Psychic, Bug,
    Rock, Ghost, Dragon, Dark, Steel, Fairy,
}

impl Type {
    /// Effectiveness against a single defending type
    pub fn effectiveness(&self, defender: Type) -> f64;

    /// Effectiveness against multiple types (multiplied)
    pub fn effectiveness_multi(&self, defenders: &[Type]) -> f64;

    /// Parse from protocol string
    pub fn from_str(s: &str) -> Option<Self>;

    /// All 18 types
    pub fn all() -> &'static [Type];
}

/// 18x18 type effectiveness chart
pub static TYPE_CHART: [[f64; 18]; 18];
```

### Status Conditions

```rust
/// Non-volatile status (persists through switch)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Status {
    Burn,
    Freeze,
    Paralysis,
    Poison,
    BadPoison,  // Toxic
    Sleep,
}

impl Status {
    pub fn from_protocol(s: &str) -> Option<Self>;
    pub fn to_protocol(&self) -> &'static str;
}

/// Volatile status (cleared on switch)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Volatile {
    Confusion,
    Taunt,
    Encore,
    Disable,
    Torment,
    Substitute,
    LeechSeed,
    Curse,
    PerishSong,
    Yawn,
    Trapped,      // Mean Look, etc.
    PartialTrap,  // Bind, Wrap, etc.
    // ... many more

    /// Unknown volatile from protocol
    Other(String),
}

impl Volatile {
    pub fn from_protocol(s: &str) -> Self;
}
```

### Stat Stages

```rust
/// Stat stages (-6 to +6)
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StatStages {
    pub atk: i8,
    pub def: i8,
    pub spa: i8,
    pub spd: i8,
    pub spe: i8,
    pub accuracy: i8,
    pub evasion: i8,
}

impl StatStages {
    /// Get stage for a stat
    pub fn get(&self, stat: Stat) -> i8;

    /// Set stage for a stat (clamped to -6..+6)
    pub fn set(&mut self, stat: Stat, value: i8);

    /// Apply a boost, returns actual change
    pub fn boost(&mut self, stat: Stat, amount: i8) -> i8;

    /// Reset all stages to 0
    pub fn clear(&mut self);

    /// Invert all stages (Topsy-Turvy)
    pub fn invert(&mut self);

    /// Get multiplier for a stat stage
    pub fn multiplier(stage: i8) -> f64 {
        // +1 = 1.5x, +2 = 2x, ..., +6 = 4x
        // -1 = 0.67x, -2 = 0.5x, ..., -6 = 0.25x
    }

    /// Get multiplier for accuracy/evasion (different formula)
    pub fn accuracy_multiplier(stage: i8) -> f64;
}
```

### Field Conditions

```rust
/// Weather conditions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Weather {
    Sun,
    Rain,
    Sand,
    Hail,
    Snow,        // Gen 9
    HarshSun,    // Primal Groudon
    HeavyRain,   // Primal Kyogre
    StrongWinds, // Mega Rayquaza
}

impl Weather {
    pub fn from_protocol(s: &str) -> Option<Self>;
    pub fn is_primal(&self) -> bool;  // Can't be overwritten
}

/// Terrain conditions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Terrain {
    Electric,
    Grassy,
    Misty,
    Psychic,
}

/// Side conditions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SideCondition {
    // Screens
    Reflect,
    LightScreen,
    AuroraVeil,

    // Entry hazards
    Spikes,       // Stackable (1-3)
    ToxicSpikes,  // Stackable (1-2)
    StealthRock,
    StickyWeb,

    // Other
    Tailwind,
    Safeguard,
    Mist,
    LuckyChant,
}

impl SideCondition {
    pub fn from_protocol(s: &str) -> Option<Self>;
    pub fn is_stackable(&self) -> bool;
    pub fn max_layers(&self) -> u8;
}
```

### Pokemon State

```rust
/// Core Pokemon identity (doesn't change during battle)
#[derive(Debug, Clone)]
pub struct PokemonIdentity {
    pub species: String,
    pub nickname: Option<String>,
    pub level: u8,
    pub gender: Option<char>,
    pub shiny: bool,
}

/// Pokemon state during battle (changes as battle progresses)
#[derive(Debug, Clone)]
pub struct PokemonState {
    // Identity
    pub identity: PokemonIdentity,

    // HP
    pub hp: u32,
    pub max_hp: Option<u32>,  // None if unknown (opponent)

    // Status
    pub status: Option<Status>,
    pub fainted: bool,
    pub active: bool,

    // Combat state
    pub boosts: StatStages,
    pub volatiles: HashSet<Volatile>,

    // Type info
    pub types: Vec<Type>,         // Current types (may change)
    pub tera_type: Option<Type>,  // If terastallized

    // Revealed information
    pub known_moves: Vec<String>,
    pub known_ability: Option<String>,
    pub known_item: Option<String>,
    pub item_consumed: bool,

    // Special states
    pub transformed: Option<String>,
}

impl PokemonState {
    pub fn new(species: &str, level: u8) -> Self;

    /// HP as percentage (0-100)
    pub fn hp_percent(&self) -> u32;

    /// Display name (nickname or species)
    pub fn name(&self) -> &str;

    /// Check for a volatile
    pub fn has_volatile(&self, v: &Volatile) -> bool;

    /// Called when switching out
    pub fn on_switch_out(&mut self);

    /// Called when switching in
    pub fn on_switch_in(&mut self);
}
```

### Side State

```rust
/// One player's side of the battle
#[derive(Debug, Clone)]
pub struct SideState {
    pub player: Player,
    pub username: String,
    pub pokemon: Vec<PokemonState>,
    pub active_slots: Vec<Option<usize>>,  // Indices into pokemon
    pub conditions: HashMap<SideCondition, SideConditionState>,
}

#[derive(Debug, Clone, Default)]
pub struct SideConditionState {
    pub layers: u8,  // For stackable conditions
}

impl SideState {
    pub fn new(player: Player, username: &str) -> Self;

    /// Get active Pokemon at slot (0 for singles)
    pub fn active(&self, slot: usize) -> Option<&PokemonState>;

    /// Get first active Pokemon (convenience for singles)
    pub fn active_pokemon(&self) -> Option<&PokemonState>;

    /// Iterate active Pokemon
    pub fn get_active(&self) -> impl Iterator<Item = &PokemonState>;

    /// Iterate bench Pokemon (not active, not fainted)
    pub fn get_bench(&self) -> impl Iterator<Item = (usize, &PokemonState)>;

    /// Count non-fainted Pokemon
    pub fn alive_count(&self) -> usize;

    /// Check for side condition
    pub fn has_condition(&self, cond: SideCondition) -> bool;

    /// Get layers for stackable condition
    pub fn condition_layers(&self, cond: SideCondition) -> u8;

    /// Add a side condition (returns false if already at max)
    pub fn add_condition(&mut self, cond: SideCondition) -> bool;

    /// Remove a side condition
    pub fn remove_condition(&mut self, cond: SideCondition);
}
```

### Field State

```rust
/// Global field state
#[derive(Debug, Clone, Default)]
pub struct FieldState {
    pub weather: Option<Weather>,
    pub terrain: Option<Terrain>,
    pub trick_room: bool,
    pub magic_room: bool,
    pub wonder_room: bool,
    pub gravity: bool,
}
```

---

## State Tracking

### TrackedBattle

The main entry point for tracking battle state from server messages.

```rust
/// A battle being tracked from server messages
#[derive(Debug, Clone)]
pub struct TrackedBattle {
    // Battle info
    pub game_type: Option<GameType>,
    pub gen: u8,
    pub tier: String,
    pub turn: u32,

    // State
    pub field: FieldState,
    pub sides: [Option<SideState>; 4],  // Up to 4 players

    // Perspective
    perspective: Option<Player>,

    // Outcome
    pub ended: bool,
    pub winner: Option<String>,
}

impl TrackedBattle {
    /// Create a new battle tracker
    pub fn new() -> Self;

    /// Set which player we are
    pub fn set_perspective(&mut self, player: Player);

    /// Get our side
    pub fn me(&self) -> Option<&SideState>;

    /// Get opponent's side (for 1v1)
    pub fn opponent(&self) -> Option<&SideState>;

    /// Get a side by player
    pub fn get_side(&self, player: Player) -> Option<&SideState>;

    /// Update from a server message
    pub fn update(&mut self, msg: &ServerMessage);

    /// Update from a request (sets perspective, full team info)
    pub fn update_from_request(&mut self, request: &BattleRequest);
}
```

### Update Logic

The `update` method handles all message types:

| Message                         | State Update                                  |
| ------------------------------- | --------------------------------------------- |
| `BattlePlayer`                  | Create side with username                     |
| `GameType`                      | Set game type                                 |
| `Gen`                           | Set generation                                |
| `Tier`                          | Set tier string                               |
| `Turn`                          | Increment turn counter                        |
| `Switch` / `Drag`               | Update active Pokemon, clear boosts/volatiles |
| `Faint`                         | Mark Pokemon fainted                          |
| `Damage` / `Heal` / `SetHp`     | Update Pokemon HP                             |
| `Status` / `CureStatus`         | Update status condition                       |
| `Boost` / `Unboost`             | Update stat stages                            |
| `ClearBoost` / `ClearAllBoost`  | Reset stat stages                             |
| `VolatileStart` / `VolatileEnd` | Add/remove volatile                           |
| `Weather`                       | Set field weather                             |
| `FieldStart` / `FieldEnd`       | Set terrain, trick room, etc.                 |
| `SideStart` / `SideEnd`         | Add/remove side conditions                    |
| `Move`                          | Record revealed move                          |
| `Ability`                       | Record revealed ability                       |
| `Item` / `EndItem`              | Record revealed/consumed item                 |
| `Win` / `Tie`                   | Mark battle ended                             |

---

## Integration with kazam-client

```rust
use kazam_client::{KazamClient, KazamHandler, KazamHandle};
use kazam_battle::{TrackedBattle, Weather, SideCondition};
use kazam_protocol::{BattleRequest, ServerMessage, RoomType, User};
use std::collections::HashMap;

struct MyBot {
    handle: KazamHandle,
    username: String,
    battles: HashMap<String, TrackedBattle>,
}

impl KazamHandler for MyBot {
    async fn on_logged_in(&mut self, user: &User) {
        self.username = user.username.clone();
    }

    async fn on_init(&mut self, room_id: &str, room_type: &RoomType) {
        if *room_type == RoomType::Battle {
            self.battles.insert(room_id.to_string(), TrackedBattle::new());
        }
    }

    async fn on_battle_message(&mut self, room_id: Option<&str>, msg: ServerMessage) {
        if let Some(battle) = room_id.and_then(|r| self.battles.get_mut(r)) {
            battle.update(&msg);
        }
    }

    async fn on_request(&mut self, room_id: &str, request: &BattleRequest) {
        let battle = match self.battles.get_mut(room_id) {
            Some(b) => b,
            None => return,
        };

        // Update with full request info (sets perspective, team details)
        battle.update_from_request(request);

        if !request.needs_decision() {
            return;
        }

        // Access tracked state
        let me = battle.me().unwrap();
        let opponent = battle.opponent().unwrap();
        let my_active = me.active_pokemon().unwrap();
        let opp_active = opponent.active_pokemon().unwrap();

        println!(
            "Turn {}: {} ({}%) vs {} ({}%)",
            battle.turn,
            my_active.name(), my_active.hp_percent(),
            opp_active.name(), opp_active.hp_percent(),
        );

        // Check field conditions
        if battle.field.weather == Some(Weather::Sun) {
            println!("  Sun is active - Fire boosted, Water weakened");
        }
        if battle.field.trick_room {
            println!("  Trick Room - slower Pokemon move first");
        }

        // Check side conditions
        if opponent.has_condition(SideCondition::Reflect) {
            println!("  Opponent has Reflect - physical damage halved");
        }
        if me.has_condition(SideCondition::StealthRock) {
            println!("  We have Stealth Rock - switch-ins take damage");
        }

        // Check Pokemon state
        if my_active.boosts.atk > 0 {
            println!("  We have +{} Attack", my_active.boosts.atk);
        }
        if my_active.has_volatile(&Volatile::Confusion) {
            println!("  We're confused!");
        }

        // Make a decision based on state
        let choice = decide_action(battle, request);
        let _ = self.handle.choose(room_id, &choice, request.rqid);
    }

    async fn on_win(&mut self, room_id: &str, winner: &str) {
        let won = winner == self.username;
        println!("Battle ended: {}", if won { "Victory!" } else { "Defeat" });
        self.battles.remove(room_id);
    }
}

fn decide_action(battle: &TrackedBattle, request: &BattleRequest) -> String {
    let me = battle.me().unwrap();
    let my_active = me.active_pokemon().unwrap();

    // Simple logic: switch if low HP, otherwise attack
    if my_active.hp_percent() < 25 && me.get_bench().count() > 0 {
        // Find healthiest bench Pokemon
        if let Some((idx, _)) = me.get_bench()
            .max_by_key(|(_, p)| p.hp_percent())
        {
            return format!("switch {}", idx + 1);
        }
    }

    // Default: use first move
    "move 1".to_string()
}
```

---

## Query Helpers

Optional utilities for decision making:

```rust
// kazam-battle/src/query/matchup.rs

/// Calculate type matchup advantage
pub fn type_advantage(attacker_types: &[Type], defender_types: &[Type]) -> f64 {
    let mut best = 1.0;
    for atk_type in attacker_types {
        let eff = atk_type.effectiveness_multi(defender_types);
        if eff > best {
            best = eff;
        }
    }
    best
}

/// Check if a Pokemon is grounded (affected by terrain, Spikes, etc.)
pub fn is_grounded(pokemon: &PokemonState, field: &FieldState) -> bool {
    // Flying type or Levitate = not grounded
    // Air Balloon = not grounded
    // Gravity = always grounded
    // etc.
}

/// Estimate speed tier (who moves first)
pub fn compare_speed(
    pokemon1: &PokemonState,
    pokemon2: &PokemonState,
    field: &FieldState,
) -> std::cmp::Ordering {
    // Account for:
    // - Base speed (if known)
    // - Speed boosts
    // - Paralysis (halves speed)
    // - Trick Room (reverses)
    // - Tailwind (doubles)
}
```

---

## Relationship with kazam-simulator

`kazam-simulator` is an **optional** crate for full battle simulation. It depends on `kazam-battle` for domain types.

| Feature               | kazam-battle             | kazam-simulator        |
| --------------------- | ------------------------ | ---------------------- |
| Type chart            | Yes                      | Reuses                 |
| Status/Volatile enums | Yes                      | Reuses                 |
| StatStages            | Yes                      | Reuses                 |
| Field/Side conditions | Yes                      | Reuses                 |
| Pokemon state         | `PokemonState` (tracked) | `BattlePokemon` (full) |
| State tracking        | Yes                      | -                      |
| Move execution        | -                        | Yes                    |
| Damage calculation    | -                        | Yes                    |
| Event system          | -                        | Yes                    |
| PRNG                  | -                        | Yes                    |
| Move/Ability data     | -                        | Yes                    |

### When to use which?

**Use `kazam-battle` only** when:

- Building a bot that reacts to server messages
- You don't need to predict future states
- You want minimal dependencies

**Add `kazam-simulator`** when:

- You need to simulate "what if" scenarios
- Building an AI that looks ahead
- Training a bot via self-play
- Writing tests for battle mechanics

### Simulator extends battle types

```rust
// In kazam-simulator, BattlePokemon wraps PokemonState with full info
pub struct BattlePokemon {
    /// Observable state (same as tracking)
    pub state: PokemonState,

    /// Full information (only simulator has this)
    pub set: PokemonSet,           // Original team set
    pub base_stats: BaseStats,     // Actual stats
    pub move_slots: Vec<MoveSlot>, // PP tracking
    pub ability_state: EffectState,
    pub item_state: EffectState,
    // ... etc
}
```

---

## Feature Flags

```toml
[features]
default = []

# Include query helpers for decision making
query = []

# Serde support for serialization
serde = ["dep:serde"]
```

---

## Dependencies

```toml
[dependencies]
kazam-protocol = { path = "../protocol" }

# Optional
serde = { version = "1.0", features = ["derive"], optional = true }

[dev-dependencies]
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

---

## Summary

`kazam-battle` is the **lightweight core** of the kazam ecosystem:

1. **Domain types** shared by tracking and simulation
2. **State tracking** from server messages
3. **No simulation overhead** - just observation
4. **Clean API** for bot decision making

For full simulation capabilities, add `kazam-simulator` which builds on these types.
