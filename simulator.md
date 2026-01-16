# kazam-simulator

A comprehensive Rust implementation of the Pokemon Showdown battle simulator. This document describes the architecture, types, and implementation strategy needed to build a full-featured battle simulator with feature parity to Pokemon Showdown.

## Goals

- **Protocol Compatibility**: Accept the same client commands and emit the same server messages as Pokemon Showdown
- **Feature Parity**: Support all battle mechanics from Gen 1-9 including:
  - Mega Evolution, Z-Moves, Dynamax/Gigantamax, Terastallization
  - All abilities, items, moves, and status conditions
  - Singles, Doubles, Triples, Multi, and Free-for-All formats
- **Integration with kazam-client**: Battle state can be reconstructed from simulator output
- **Deterministic**: Seeded PRNG for reproducible battles and replays
- **Type-Safe**: Complete Rust type coverage for all game mechanics
- **Modular**: Clean separation between data, logic, and I/O

---

## Relationship with kazam-battle

The simulator **depends on `kazam-battle`** for shared domain types. This avoids type duplication and ensures compatibility between state tracking and simulation.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           Dependency Graph                                   │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   kazam-protocol       (wire format: ServerMessage, BattleRequest)          │
│         │                                                                    │
│         ▼                                                                    │
│   kazam-battle         (domain types + state tracking)                       │
│         │              - Type, Status, Volatile, StatStages                  │
│         │              - Weather, Terrain, SideCondition                     │
│         │              - PokemonState, SideState, FieldState                 │
│         │              - TrackedBattle (for observing real battles)          │
│         │                                                                    │
│         ▼                                                                    │
│   kazam-simulator      (full simulation - THIS CRATE)                        │
│                        - Extends kazam-battle types                          │
│                        - BattlePokemon wraps PokemonState                    │
│                        - Adds: Dex, event system, damage calc, PRNG          │
│                        - Adds: PokemonSet, Team, Move execution              │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### What kazam-battle provides (reused by simulator)

| Type            | Description                            |
| --------------- | -------------------------------------- |
| `Type`          | Pokemon types with effectiveness chart |
| `Status`        | Non-volatile status conditions         |
| `Volatile`      | Volatile status conditions             |
| `StatStages`    | Stat stage modifiers (-6 to +6)        |
| `Weather`       | Weather conditions                     |
| `Terrain`       | Terrain conditions                     |
| `SideCondition` | Side conditions (hazards, screens)     |
| `PokemonState`  | Observable Pokemon state               |
| `SideState`     | Observable side state                  |
| `FieldState`    | Observable field state                 |

### What kazam-simulator adds

| Type                                 | Description                                 |
| ------------------------------------ | ------------------------------------------- |
| `BattlePokemon`                      | Full Pokemon state (extends `PokemonState`) |
| `BattleSide`                         | Full side state (extends `SideState`)       |
| `PokemonSet`                         | Team builder format                         |
| `Move`, `Ability`, `Item`, `Species` | Game data with effect handlers              |
| `Dex`                                | Data lookup system                          |
| `Event`, `EventListener`             | Event system for mechanics                  |
| `Prng`                               | Deterministic RNG                           |
| `BattleStream`                       | Protocol I/O                                |

### Type Extension Pattern

The simulator extends kazam-battle's observable state with full information:

```rust
// kazam-battle provides observable state
pub use kazam_battle::{
    Type, Status, Volatile, StatStages,
    Weather, Terrain, SideCondition,
    PokemonState, SideState, FieldState,
};

// kazam-simulator extends with full state
pub struct BattlePokemon {
    /// Observable state (same as tracking would see)
    pub state: PokemonState,

    /// Full information only simulator has:
    pub set: PokemonSet,              // Original team set
    pub base_stats: BaseStats,        // Actual base stats
    pub stored_stats: CalculatedStats, // Calculated stats
    pub move_slots: Vec<MoveSlot>,    // PP tracking
    pub base_ability: AbilityId,      // Original ability
    pub ability_state: EffectState,   // Ability volatile data
    pub item_state: EffectState,      // Item volatile data
    // ... etc
}
```

This design means:

1. **Simulator output is compatible with tracking** - Protocol messages can be fed to `TrackedBattle`
2. **No type duplication** - Domain types defined once in kazam-battle
3. **Clear separation** - Tracking doesn't need simulation overhead

---

## Architecture Overview

```
                    ┌─────────────────────────────────────────────────────────┐
                    │                    BattleStream                         │
                    │  (I/O layer - converts text protocol to/from types)     │
                    └─────────────────────────────────────────────────────────┘
                                              │
                    ┌─────────────────────────┴───────────────────────────────┐
                    │                       Battle                            │
                    │            (Main orchestrator & event system)           │
                    └─────────────────────────────────────────────────────────┘
                         │              │              │              │
            ┌────────────┴───┐    ┌────┴────┐    ┌───┴───┐    ┌─────┴─────┐
            │  BattleQueue   │    │  Field  │    │ Side  │    │  Actions  │
            │ (Priority sort)│    │(Weather)│    │(Team) │    │(Mechanics)│
            └────────────────┘    └─────────┘    └───────┘    └───────────┘
                                                      │
                                                 ┌────┴────┐
                                                 │ Pokemon │
                                                 │ (State) │
                                                 └─────────┘
                    ┌─────────────────────────────────────────────────────────┐
                    │                         Dex                             │
                    │    (Data lookup: Species, Moves, Abilities, Items)      │
                    └─────────────────────────────────────────────────────────┘
```

---

## Core Types

### Reused from kazam-protocol

Wire format types for protocol parsing/serialization:

```rust
pub use kazam_protocol::{
    Player,           // p1, p2, p3, p4
    Pokemon,          // Position + name identifier (e.g., "p1a: Pikachu")
    PokemonDetails,   // Species, level, gender, shiny, tera_type
    HpStatus,         // Current/max HP + status condition
    Side,             // Player reference for side conditions
    Stat,             // atk, def, spa, spd, spe, accuracy, evasion
    GameType,         // singles, doubles, triples, multi, freeforall
};
```

### Reused from kazam-battle

Domain types shared with state tracking:

```rust
pub use kazam_battle::{
    // Type system
    Type,             // Pokemon types (Normal, Fire, Water, ...)
    TYPE_CHART,       // 18x18 effectiveness chart

    // Status conditions
    Status,           // Non-volatile (Burn, Freeze, Paralysis, ...)
    Volatile,         // Volatile (Confusion, Taunt, Substitute, ...)

    // Stats
    StatStages,       // Stat modifiers (-6 to +6)

    // Field conditions
    Weather,          // Sun, Rain, Sand, etc.
    Terrain,          // Electric, Grassy, Misty, Psychic
    SideCondition,    // Reflect, Spikes, Stealth Rock, etc.

    // Observable state (extended by simulator)
    PokemonState,     // Pokemon state as observed from protocol
    SideState,        // Side state as observed from protocol
    FieldState,       // Field state as observed from protocol
};
```

### Simulator-Only Types

The following types are **only in kazam-simulator** (not needed for tracking):

#### Extended Type System

The simulator extends the `Type` enum from kazam-battle with additional methods:

```rust
/// Effectiveness result (combines immunity check with multiplier)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Effectiveness {
    Immune,        // 0x
    NotVeryX2,     // 0.25x (double resistance)
    NotVery,       // 0.5x
    Normal,        // 1x
    SuperX2,       // 2x
    SuperX4,       // 4x (double weakness)
}

impl Effectiveness {
    pub fn multiplier(&self) -> f64 {
        match self {
            Effectiveness::Immune => 0.0,
            Effectiveness::NotVeryX2 => 0.25,
            Effectiveness::NotVery => 0.5,
            Effectiveness::Normal => 1.0,
            Effectiveness::SuperX2 => 2.0,
            Effectiveness::SuperX4 => 4.0,
        }
    }
}

/// Extension trait for Type (from kazam-battle)
impl Type {
    /// Get type effectiveness against a single type
    /// (Uses TYPE_CHART from kazam-battle)
    pub fn effectiveness_against(&self, defender: Type) -> Effectiveness {
        TYPE_CHART[*self as usize][defender as usize]
    }

    /// Get combined effectiveness against multiple types
    pub fn effectiveness_against_types(&self, defenders: &[Type]) -> f64 {
        defenders.iter()
            .map(|t| self.effectiveness_against(*t).multiplier())
            .product()
    }
}
```

### Status Conditions

```rust
/// Non-volatile status conditions (persist after switching)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Status {
    Burn,
    Freeze,
    Paralysis,
    Poison,
    BadPoison,    // Toxic
    Sleep,
}

impl Status {
    /// Parse from protocol string ("brn", "frz", "par", "psn", "tox", "slp")
    pub fn from_str(s: &str) -> Option<Self>;

    /// Convert to protocol string
    pub fn as_str(&self) -> &'static str;
}

/// Volatile status conditions (cleared on switch)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Volatile {
    // Movement restriction
    Bound,          // Bind, Wrap, Fire Spin, etc.
    CantEscape,     // Mean Look, Spider Web
    Trapped,        // Block, Shadow Tag, Arena Trap

    // Attack modification
    Confusion,
    Curse,          // Ghost-type Curse
    Disable,
    Encore,
    HealBlock,
    Imprison,
    Infatuation,    // Attract
    LeechSeed,
    Nightmare,
    PerishSong,
    Taunt,
    Torment,

    // Stat-related
    FlashFire,      // Ability-granted immunity

    // Protection
    Protect,
    Endure,

    // Turn-based states
    Charging,       // Solar Beam, Sky Attack, etc.
    Recharging,     // Hyper Beam, Giga Impact
    Roosting,       // Roost (loses Flying type)

    // Transformation
    Transformed,    // Transform/Imposter
    Illusion,       // Zoroark's Illusion

    // Multi-turn moves
    Bide,
    Thrash,         // Outrage, Petal Dance, etc.
    Rollout,
    IceBall,
    Uproar,

    // Gen 6+ mechanics
    Substitute,

    // Gen 8+ mechanics
    Dynamaxed,

    // Gen 9+ mechanics
    Terastallized,

    // Misc
    Minimize,       // Stomp does double damage
    DefenseCurl,    // Rollout does double damage
    Flinch,
    FocusEnergy,
    GastroAcid,     // Ability suppressed
    Foresight,      // Ghost hit by Normal/Fighting
    MiracleEye,     // Dark hit by Psychic
    Smacked,        // Hit this turn (for Counter/Mirror Coat)
    MustRecharge,
}

/// State for a volatile condition (duration, source, etc.)
#[derive(Debug, Clone)]
pub struct VolatileState {
    pub id: Volatile,
    pub source: Option<PokemonRef>,
    pub source_effect: Option<EffectId>,
    pub duration: Option<u8>,
    pub counter: Option<u8>,
    /// Arbitrary data (e.g., disabled move, encore move)
    pub data: Option<VolatileData>,
}

#[derive(Debug, Clone)]
pub enum VolatileData {
    Move(MoveId),
    Pokemon(PokemonRef),
    Damage(u32),
    Custom(String),
}
```

### Stat Modifiers

```rust
/// Stat stages (-6 to +6)
#[derive(Debug, Clone, Default)]
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
    pub fn get(&self, stat: Stat) -> i8;
    pub fn set(&mut self, stat: Stat, value: i8);
    pub fn boost(&mut self, stat: Stat, amount: i8) -> i8;  // Returns actual change
    pub fn clear(&mut self);
    pub fn invert(&mut self);

    /// Get the multiplier for a stat stage
    pub fn multiplier(stage: i8) -> f64 {
        match stage.clamp(-6, 6) {
            -6 => 2.0 / 8.0,
            -5 => 2.0 / 7.0,
            -4 => 2.0 / 6.0,
            -3 => 2.0 / 5.0,
            -2 => 2.0 / 4.0,
            -1 => 2.0 / 3.0,
            0 => 1.0,
            1 => 3.0 / 2.0,
            2 => 4.0 / 2.0,
            3 => 5.0 / 2.0,
            4 => 6.0 / 2.0,
            5 => 7.0 / 2.0,
            6 => 8.0 / 2.0,
            _ => unreachable!(),
        }
    }

    /// Get the multiplier for accuracy/evasion stages
    pub fn accuracy_multiplier(stage: i8) -> f64 {
        match stage.clamp(-6, 6) {
            -6 => 3.0 / 9.0,
            -5 => 3.0 / 8.0,
            -4 => 3.0 / 7.0,
            -3 => 3.0 / 6.0,
            -2 => 3.0 / 5.0,
            -1 => 3.0 / 4.0,
            0 => 1.0,
            1 => 4.0 / 3.0,
            2 => 5.0 / 3.0,
            3 => 6.0 / 3.0,
            4 => 7.0 / 3.0,
            5 => 8.0 / 3.0,
            6 => 9.0 / 3.0,
            _ => unreachable!(),
        }
    }
}

/// Base stats for a Pokemon species
#[derive(Debug, Clone)]
pub struct BaseStats {
    pub hp: u32,
    pub atk: u32,
    pub def: u32,
    pub spa: u32,
    pub spd: u32,
    pub spe: u32,
}

/// Calculated stats for a specific Pokemon
#[derive(Debug, Clone)]
pub struct CalculatedStats {
    pub hp: u32,
    pub atk: u32,
    pub def: u32,
    pub spa: u32,
    pub spd: u32,
    pub spe: u32,
}

impl CalculatedStats {
    /// Calculate stats from base stats, IVs, EVs, level, and nature
    pub fn calculate(
        base: &BaseStats,
        ivs: &IndividualValues,
        evs: &EffortValues,
        level: u8,
        nature: Nature,
    ) -> Self;
}
```

### Individual Values and Effort Values

```rust
#[derive(Debug, Clone, Default)]
pub struct IndividualValues {
    pub hp: u8,   // 0-31
    pub atk: u8,
    pub def: u8,
    pub spa: u8,
    pub spd: u8,
    pub spe: u8,
}

#[derive(Debug, Clone, Default)]
pub struct EffortValues {
    pub hp: u8,   // 0-252, total max 510
    pub atk: u8,
    pub def: u8,
    pub spa: u8,
    pub spd: u8,
    pub spe: u8,
}
```

### Nature

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Nature {
    Hardy, Lonely, Brave, Adamant, Naughty,
    Bold, Docile, Relaxed, Impish, Lax,
    Timid, Hasty, Serious, Jolly, Naive,
    Modest, Mild, Quiet, Bashful, Rash,
    Calm, Gentle, Sassy, Careful, Quirky,
}

impl Nature {
    /// Get the stat increased by this nature (None for neutral natures)
    pub fn increased_stat(&self) -> Option<Stat>;

    /// Get the stat decreased by this nature (None for neutral natures)
    pub fn decreased_stat(&self) -> Option<Stat>;

    /// Get the multiplier for a stat (1.0, 1.1, or 0.9)
    pub fn stat_multiplier(&self, stat: Stat) -> f64;
}
```

---

## Pokemon Set (Team Builder Format)

```rust
/// A Pokemon set as specified in team building
#[derive(Debug, Clone)]
pub struct PokemonSet {
    /// Nickname (or species name if none)
    pub name: String,

    /// Species (including forme, e.g., "Pikachu-Alola")
    pub species: SpeciesId,

    /// Held item
    pub item: Option<ItemId>,

    /// Ability
    pub ability: AbilityId,

    /// Moves (1-4)
    pub moves: Vec<MoveId>,

    /// Nature
    pub nature: Nature,

    /// Gender (for species that can have either)
    pub gender: Option<Gender>,

    /// Individual values
    pub ivs: IndividualValues,

    /// Effort values
    pub evs: EffortValues,

    /// Level (1-100, default 100)
    pub level: u8,

    /// Shiny
    pub shiny: bool,

    /// Happiness (0-255, default 255)
    pub happiness: u8,

    /// Pokeball
    pub pokeball: String,

    /// Hidden Power type (calculated from IVs, but can be overridden)
    pub hp_type: Option<Type>,

    /// Dynamax level (0-10, Gen 8)
    pub dynamax_level: u8,

    /// Can Gigantamax (Gen 8)
    pub gigantamax: bool,

    /// Tera type (Gen 9)
    pub tera_type: Option<Type>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gender {
    Male,
    Female,
    Genderless,
}

impl PokemonSet {
    /// Parse from packed team format
    pub fn from_packed(packed: &str) -> Result<Self, ParseError>;

    /// Serialize to packed team format
    pub fn to_packed(&self) -> String;

    /// Parse from JSON format
    pub fn from_json(json: &serde_json::Value) -> Result<Self, ParseError>;

    /// Serialize to JSON format
    pub fn to_json(&self) -> serde_json::Value;
}
```

### Team Format

```rust
/// A full team of Pokemon
pub type Team = Vec<PokemonSet>;

/// Parse/serialize teams
pub fn parse_team(packed: &str) -> Result<Team, ParseError>;
pub fn pack_team(team: &Team) -> String;

/// Packed format:
/// "NAME|SPECIES|ITEM|ABILITY|MOVE1,MOVE2,MOVE3,MOVE4|NATURE|EVS|IVS|GENDER|LEVEL|SHINY|HAPPINESS]..."
/// Fields separated by |, Pokemon separated by ]
```

---

## Battle Pokemon (In-Battle State)

```rust
/// A Pokemon's state during battle
pub struct BattlePokemon {
    // === Identity (from PokemonSet) ===

    /// Original set this Pokemon was created from
    pub set: PokemonSet,

    /// Position in party (0-5)
    pub position: usize,

    /// Owning side
    pub side_id: SideId,

    // === Species/Forme ===

    /// Base species (doesn't change even with Transform)
    pub base_species: &'static Species,

    /// Current species (changes with forme changes, Transform)
    pub species: &'static Species,

    // === HP and Status ===

    /// Current HP
    pub hp: u32,

    /// Maximum HP (changes with Dynamax)
    pub max_hp: u32,

    /// Status condition (None if healthy)
    pub status: Option<Status>,

    /// Fainted flag
    pub fainted: bool,

    // === Stats ===

    /// Base calculated stats (before battle modifiers)
    pub base_stored_stats: CalculatedStats,

    /// Stored stats (after Transform, etc.)
    pub stored_stats: CalculatedStats,

    /// Stat stages (-6 to +6)
    pub boosts: StatStages,

    // === Ability and Item ===

    /// Base ability (permanent)
    pub base_ability: AbilityId,

    /// Current ability (can change via Skill Swap, etc.)
    pub ability: AbilityId,

    /// Ability state (volatile data for ability effects)
    pub ability_state: EffectState,

    /// Current held item
    pub item: Option<ItemId>,

    /// Item state (volatile data for item effects)
    pub item_state: EffectState,

    /// Last item held (for Recycle, Unburden, etc.)
    pub last_item: Option<ItemId>,

    /// Item consumed this turn (for Unburden)
    pub used_item_this_turn: bool,

    // === Moves ===

    /// Move slots with current PP
    pub move_slots: Vec<MoveSlot>,

    /// Base moves (for Transform)
    pub base_moves: Vec<MoveId>,

    /// Last move used
    pub last_move: Option<ActiveMove>,

    /// Last move used this turn
    pub last_move_used: Option<ActiveMove>,

    /// Whether a move was used this turn
    pub move_this_turn: MoveThisTurn,

    /// Result of last move
    pub move_this_turn_result: Option<MoveResult>,

    /// Result of last turn's move
    pub move_last_turn_result: Option<MoveResult>,

    // === Volatile State ===

    /// Active volatile conditions
    pub volatiles: HashMap<Volatile, VolatileState>,

    /// Whether this Pokemon has Transformed
    pub transformed: bool,

    /// Illusion target (for Zoroark)
    pub illusion: Option<PokemonRef>,

    // === Battle Tracking ===

    /// Turn this Pokemon was switched in (0 if never active)
    pub switch_in_turn: u32,

    /// Number of turns active on the field
    pub active_turns: u32,

    /// Number of move actions this switch-in (for Fake Out)
    pub active_move_actions: u32,

    /// Whether newly switched in this turn
    pub newly_switched: bool,

    /// Turn this Pokemon was dragged/forced out (for Pursuit)
    pub being_called_back: Option<u32>,

    // === Damage Tracking ===

    /// Times attacked this turn
    pub times_attacked: u32,

    /// Attackers and damage this turn
    pub attacked_by: Vec<AttackInfo>,

    /// Total damage dealt this turn
    pub total_damage_dealt: u32,

    // === Special Mechanics ===

    /// Tera type if terastallized
    pub terastallized: Option<Type>,

    /// Dynamax turns remaining (0 if not dynamaxed)
    pub dynamax_turns: u8,

    /// Whether mega evolved this battle
    pub mega_evolved: bool,

    /// Number of times status was inflicted (for Early Bird)
    pub status_data: StatusData,
}

#[derive(Debug, Clone)]
pub struct MoveSlot {
    pub move_id: MoveId,
    pub pp: u32,
    pub max_pp: u32,
    /// Whether PP was used this turn
    pub used: bool,
    /// Virtual flag (for Mimic, Sketch)
    pub is_virtual: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum MoveThisTurn {
    None,
    About,      // About to move
    Used,       // Move was used
}

#[derive(Debug, Clone, Copy)]
pub enum MoveResult {
    Success,
    Failed,
    Missed,
    Blocked,
}

#[derive(Debug, Clone)]
pub struct AttackInfo {
    pub source: PokemonRef,
    pub damage: u32,
    pub source_effect: EffectId,
    pub this_turn: bool,
}

#[derive(Debug, Clone, Default)]
pub struct StatusData {
    /// Turns remaining for Sleep
    pub sleep_turns: u8,
    /// Toxic counter
    pub toxic_turns: u8,
}

impl BattlePokemon {
    /// Create a new battle Pokemon from a set
    pub fn new(set: &PokemonSet, position: usize, side_id: SideId, dex: &Dex) -> Self;

    /// Get effective speed (after modifiers)
    pub fn get_speed(&self, battle: &Battle) -> u32;

    /// Get effective stat (with boosts and abilities)
    pub fn get_stat(&self, stat: Stat, battle: &Battle) -> u32;

    /// Check if this Pokemon has a volatile
    pub fn has_volatile(&self, volatile: Volatile) -> bool;

    /// Add a volatile condition
    pub fn add_volatile(&mut self, volatile: Volatile, state: VolatileState);

    /// Remove a volatile condition
    pub fn remove_volatile(&mut self, volatile: Volatile) -> Option<VolatileState>;

    /// Get current types (affected by type-changing effects)
    pub fn get_types(&self) -> Vec<Type>;

    /// Check if the Pokemon has a specific type
    pub fn has_type(&self, typ: Type) -> bool;

    /// Check if active
    pub fn is_active(&self) -> bool;

    /// Get HP as fraction for protocol
    pub fn get_hp_string(&self, hidden: bool) -> String;

    /// Get details string for protocol
    pub fn get_details_string(&self) -> String;

    /// Get identifier string for protocol
    pub fn get_ident_string(&self, position: Option<char>) -> String;
}
```

---

## Side (Player State)

```rust
/// One player's side of the battle
pub struct BattleSide {
    /// Side identifier
    pub id: SideId,

    /// Player number (P1, P2, P3, P4)
    pub player: Player,

    /// Player's display name
    pub name: String,

    /// Player's avatar
    pub avatar: String,

    /// Player's rating (if rated battle)
    pub rating: Option<u32>,

    /// Original team (for reference)
    pub team: Vec<PokemonSet>,

    /// Pokemon on this side
    pub pokemon: Vec<BattlePokemon>,

    /// Currently active Pokemon indices (1 for singles, 2 for doubles, etc.)
    pub active: Vec<Option<usize>>,

    /// Number of Pokemon not yet fainted
    pub pokemon_left: u8,

    /// Side conditions (Reflect, Spikes, etc.)
    pub side_conditions: HashMap<SideCondition, SideConditionState>,

    /// Slot conditions (Wish, Future Sight targets)
    pub slot_conditions: Vec<HashMap<SlotCondition, SlotConditionState>>,

    /// Active choice request
    pub request: Option<ChoiceRequest>,

    /// Current choice being built
    pub choice: Choice,

    /// Last pokemon switched out (for Pursuit)
    pub last_switched_out: Option<PokemonRef>,

    // === Battle-Wide Tracking ===

    /// Pokemon that fainted last turn
    pub fainted_last_turn: Option<PokemonRef>,

    /// Pokemon that fainted this turn
    pub fainted_this_turn: Option<PokemonRef>,

    /// Total faints this battle
    pub total_fainted: u32,

    // === Mechanic Flags ===

    /// Z-move used this battle
    pub z_move_used: bool,

    /// Dynamax used this battle
    pub dynamax_used: bool,

    /// Mega evolved this battle
    pub mega_evolved: bool,

    /// Terastallized this battle
    pub terastallized: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SideCondition {
    // Entry hazards
    Spikes,
    ToxicSpikes,
    StealthRock,
    StickyWeb,

    // Screens
    Reflect,
    LightScreen,
    AuroraVeil,

    // Other
    Tailwind,
    Safeguard,
    Mist,
    LuckyChant,

    // Gen 8
    GMaxWildfire,
    GMaxVolcalith,
    GMaxVineLash,
    GMaxCannonade,
    GMaxSteelsurge,
}

#[derive(Debug, Clone)]
pub struct SideConditionState {
    pub id: SideCondition,
    pub source: Option<PokemonRef>,
    pub duration: Option<u8>,
    pub layers: u8,  // For Spikes, Toxic Spikes
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SlotCondition {
    Wish,
    HealingWish,
    LunarDance,
    FutureSight,
    DoomDesire,
}

#[derive(Debug, Clone)]
pub struct SlotConditionState {
    pub id: SlotCondition,
    pub source: PokemonRef,
    pub turn: u32,
    pub damage: Option<u32>,
    pub hp: Option<u32>,
}

impl BattleSide {
    /// Get the active Pokemon at a position
    pub fn active_pokemon(&self, pos: usize) -> Option<&BattlePokemon>;

    /// Get all active Pokemon
    pub fn get_active(&self) -> impl Iterator<Item = &BattlePokemon>;

    /// Get benched (non-active, non-fainted) Pokemon
    pub fn get_bench(&self) -> impl Iterator<Item = &BattlePokemon>;

    /// Get all non-fainted Pokemon
    pub fn get_alive(&self) -> impl Iterator<Item = &BattlePokemon>;

    /// Check if all Pokemon have fainted
    pub fn all_fainted(&self) -> bool;

    /// Get a Pokemon by position in party
    pub fn get_pokemon(&self, index: usize) -> Option<&BattlePokemon>;

    /// Has side condition
    pub fn has_side_condition(&self, condition: SideCondition) -> bool;

    /// Add side condition
    pub fn add_side_condition(&mut self, condition: SideCondition, state: SideConditionState) -> bool;

    /// Remove side condition
    pub fn remove_side_condition(&mut self, condition: SideCondition) -> Option<SideConditionState>;

    /// Make a choice for this side
    pub fn choose(&mut self, choice_str: &str) -> Result<(), ChoiceError>;

    /// Clear the current choice
    pub fn clear_choice(&mut self);

    /// Check if choice is complete
    pub fn is_choice_complete(&self) -> bool;

    /// Generate the request JSON
    pub fn make_request(&self, battle: &Battle) -> serde_json::Value;
}
```

---

## Field (Global Battle State)

```rust
/// Global field conditions
pub struct Field {
    /// Current weather
    pub weather: Option<Weather>,
    pub weather_state: WeatherState,

    /// Current terrain
    pub terrain: Option<Terrain>,
    pub terrain_state: TerrainState,

    /// Pseudo-weather (Trick Room, Magic Room, etc.)
    pub pseudo_weather: HashMap<PseudoWeather, PseudoWeatherState>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Weather {
    Sun,
    Rain,
    Sand,
    Hail,
    Snow,           // Gen 9 replacement for Hail
    HarshSun,       // Primal Groudon
    HeavyRain,      // Primal Kyogre
    StrongWinds,    // Mega Rayquaza
}

#[derive(Debug, Clone)]
pub struct WeatherState {
    pub weather: Weather,
    pub source: Option<PokemonRef>,
    pub duration: Option<u8>,  // None = infinite (from ability)
    pub source_effect: Option<EffectId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Terrain {
    Electric,
    Grassy,
    Misty,
    Psychic,
}

#[derive(Debug, Clone)]
pub struct TerrainState {
    pub terrain: Terrain,
    pub source: Option<PokemonRef>,
    pub duration: Option<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PseudoWeather {
    TrickRoom,
    MagicRoom,
    WonderRoom,
    Gravity,
    MudSport,
    WaterSport,
    IonDeluge,
    FairyLock,
}

#[derive(Debug, Clone)]
pub struct PseudoWeatherState {
    pub id: PseudoWeather,
    pub source: Option<PokemonRef>,
    pub duration: Option<u8>,
}

impl Field {
    pub fn new() -> Self;

    /// Set the weather
    pub fn set_weather(&mut self, weather: Weather, state: WeatherState) -> bool;

    /// Clear the weather
    pub fn clear_weather(&mut self);

    /// Check if weather is active (not suppressed)
    pub fn is_weather_active(&self, battle: &Battle) -> bool;

    /// Set the terrain
    pub fn set_terrain(&mut self, terrain: Terrain, state: TerrainState) -> bool;

    /// Clear the terrain
    pub fn clear_terrain(&mut self);

    /// Add pseudo-weather
    pub fn add_pseudo_weather(&mut self, pw: PseudoWeather, state: PseudoWeatherState) -> bool;

    /// Remove pseudo-weather
    pub fn remove_pseudo_weather(&mut self, pw: PseudoWeather);

    /// Check if Trick Room is active
    pub fn is_trick_room(&self) -> bool;

    /// Check if Gravity is active
    pub fn is_gravity(&self) -> bool;
}
```

---

## Actions and Choice System

### Player Choices

```rust
/// A player's choice for one active Pokemon
#[derive(Debug, Clone)]
pub enum PokemonChoice {
    /// Use a move
    Move {
        move_index: usize,      // 0-3
        target: Option<i8>,     // Target slot (-2 to +2), None for self/field
        mega: bool,
        zmove: bool,
        dynamax: bool,
        terastallize: bool,
    },

    /// Switch to a different Pokemon
    Switch {
        pokemon_index: usize,   // Index in party
    },

    /// Pass (for fainted Pokemon in doubles)
    Pass,

    /// Shift position (triples only)
    Shift,
}

/// A full choice for one side
#[derive(Debug, Clone)]
pub struct Choice {
    /// Choices for each active slot
    pub actions: Vec<PokemonChoice>,

    /// Z-move selected for this turn
    pub z_move: bool,

    /// Mega evolution selected for this turn
    pub mega: bool,

    /// Dynamax selected for this turn
    pub dynamax: bool,

    /// Terastallization selected for this turn
    pub terastallize: bool,

    /// Number of forced switches remaining
    pub forced_switches_left: u8,
}

/// Parse a choice string into a Choice struct
pub fn parse_choice(choice_str: &str, request: &ChoiceRequest) -> Result<Choice, ChoiceError>;

#[derive(Debug, Clone)]
pub enum ChoiceError {
    InvalidFormat(String),
    InvalidMove(String),
    InvalidSwitch(String),
    InvalidTarget(String),
    Disabled(String),
    Trapped,
    NoTarget,
}
```

### Actions

```rust
/// A resolved action to be executed
#[derive(Debug, Clone)]
pub enum Action {
    /// Use a move
    Move(MoveAction),

    /// Switch Pokemon
    Switch(SwitchAction),

    /// Team preview reordering
    Team(TeamAction),

    /// Field-wide action (residual, etc.)
    Field(FieldAction),

    /// Pokemon-specific action (mega evo, before move)
    Pokemon(PokemonAction),
}

#[derive(Debug, Clone)]
pub struct MoveAction {
    pub pokemon: PokemonRef,
    pub move_id: MoveId,
    pub target: Option<PokemonRef>,
    pub original_target: Option<PokemonRef>,
    pub priority: i8,
    pub speed: u32,
    pub mega: bool,
    pub zmove: bool,
    pub dynamax: bool,
    pub order: u32,
}

#[derive(Debug, Clone)]
pub struct SwitchAction {
    pub pokemon: PokemonRef,
    pub target_index: usize,
    pub priority: i8,
    pub speed: u32,
    pub order: u32,
}

#[derive(Debug, Clone)]
pub struct TeamAction {
    pub side: SideId,
    pub slots: Vec<usize>,
}

#[derive(Debug, Clone)]
pub struct FieldAction {
    pub action_type: FieldActionType,
    pub order: u32,
}

#[derive(Debug, Clone)]
pub enum FieldActionType {
    /// Start of battle
    Start,
    /// Before turn actions
    BeforeTurn,
    /// After turn actions (residual)
    Residual,
}

#[derive(Debug, Clone)]
pub struct PokemonAction {
    pub pokemon: PokemonRef,
    pub action_type: PokemonActionType,
    pub order: u32,
}

#[derive(Debug, Clone)]
pub enum PokemonActionType {
    MegaEvo,
    UltraBurst,
    Terastallize,
    RunSwitch,
    BeforeMove,
}
```

---

## Battle Queue (Action Ordering)

```rust
/// The battle queue manages action ordering
pub struct BattleQueue {
    /// Pending actions to execute
    pub list: Vec<Action>,
}

impl BattleQueue {
    pub fn new() -> Self;

    /// Add an action to the queue
    pub fn push(&mut self, action: Action);

    /// Get the next action
    pub fn pop(&mut self) -> Option<Action>;

    /// Peek at the next action
    pub fn peek(&self) -> Option<&Action>;

    /// Check if queue is empty
    pub fn is_empty(&self) -> bool;

    /// Sort the queue by priority
    pub fn sort(&mut self, prng: &mut Prng);

    /// Resolve a choice into actions
    pub fn resolve_action(
        &self,
        choice: &PokemonChoice,
        pokemon: &BattlePokemon,
        battle: &Battle,
    ) -> Vec<Action>;

    /// Clear the queue
    pub fn clear(&mut self);

    /// Insert an action with specific priority (for Pursuit, etc.)
    pub fn insert_choice(&mut self, action: Action, relative_to: Option<&Action>);
}

/// Compare two actions for priority
/// Order: order → priority → fractional_priority → speed → subOrder → tie_break
pub fn compare_priority(a: &Action, b: &Action, prng: &mut Prng) -> Ordering {
    // 1. Compare order (lower = first)
    //    - 3: instant actions
    //    - 5: beforeTurn
    //    - 200: normal moves
    //    - 300: residual

    // 2. Compare priority (higher = first)
    //    - +5: Helping Hand
    //    - +4: Protect, Detect
    //    - +3: Fake Out
    //    - +2: Extreme Speed, Follow Me
    //    - +1: Quick Attack, Mach Punch
    //    - 0: most moves
    //    - -1: Vital Throw
    //    - -6: Whirlwind
    //    - -7: Trick Room

    // 3. Compare speed (higher = first, unless Trick Room)

    // 4. Fischer-Yates shuffle for ties
}
```

---

## Battle (Main Orchestrator)

```rust
/// The main battle struct
pub struct Battle {
    // === Configuration ===

    /// Format being played
    pub format: FormatId,

    /// Game type (singles, doubles, etc.)
    pub game_type: GameType,

    /// Generation
    pub gen: u8,

    /// Rules and clauses
    pub rules: Vec<Rule>,

    /// Whether this is a rated battle
    pub rated: bool,

    /// Format tier name
    pub tier: String,

    // === State ===

    /// Player sides
    pub sides: Vec<BattleSide>,

    /// Field conditions
    pub field: Field,

    /// Action queue
    pub queue: BattleQueue,

    /// Battle actions executor
    pub actions: BattleActions,

    /// Current turn number (0 = not started, 1+ = in progress)
    pub turn: u32,

    /// Current request state
    pub request_state: RequestState,

    /// Winner (None if not ended)
    pub winner: Option<SideId>,

    /// Ended in tie
    pub tie: bool,

    /// Battle has started
    pub started: bool,

    /// Battle has ended
    pub ended: bool,

    // === PRNG ===

    /// Random number generator
    pub prng: Prng,

    /// Original seed (for replay)
    pub seed: PrngSeed,

    // === Logging ===

    /// Message log (protocol output)
    pub log: Vec<String>,

    /// Input log (for replay)
    pub input_log: Vec<String>,

    // === References ===

    /// Dex reference for data lookup
    pub dex: Arc<Dex>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequestState {
    None,
    TeamPreview,
    Move,
    Switch,
}

impl Battle {
    // === Construction ===

    /// Create a new battle
    pub fn new(options: BattleOptions, dex: Arc<Dex>) -> Self;

    /// Set a player
    pub fn set_player(&mut self, player: Player, options: PlayerOptions) -> Result<(), BattleError>;

    // === Turn Execution ===

    /// Start the battle (after both players set)
    pub fn start(&mut self);

    /// Make a choice for a player
    pub fn choose(&mut self, player: Player, choice: &str) -> Result<(), ChoiceError>;

    /// Commit choices and run the turn
    pub fn commit_choices(&mut self);

    /// Run one turn of the battle
    pub fn run_turn(&mut self);

    /// End the battle
    pub fn end(&mut self, winner: Option<SideId>, tie: bool);

    // === Event System ===

    /// Run an event through all listeners
    pub fn run_event<T: EventResult>(
        &mut self,
        event_id: EventId,
        source: Option<PokemonRef>,
        target: Option<PokemonRef>,
        effect: Option<&Effect>,
        relay_var: T,
    ) -> T;

    /// Run an event that expects a boolean result
    pub fn run_event_bool(
        &mut self,
        event_id: EventId,
        source: Option<PokemonRef>,
        target: Option<PokemonRef>,
        effect: Option<&Effect>,
    ) -> bool;

    /// Run an event on all active Pokemon in speed order
    pub fn each_event(&mut self, event_id: EventId);

    // === Helpers ===

    /// Get a side by ID
    pub fn get_side(&self, side_id: SideId) -> &BattleSide;

    /// Get a side mutably by ID
    pub fn get_side_mut(&mut self, side_id: SideId) -> &mut BattleSide;

    /// Get the opposing side
    pub fn get_opponent(&self, side_id: SideId) -> &BattleSide;

    /// Get all active Pokemon in speed order
    pub fn get_all_active(&self) -> Vec<PokemonRef>;

    /// Speed sort a list of Pokemon
    pub fn speed_sort(&self, pokemon: &mut [PokemonRef]);

    /// Random chance check
    pub fn random_chance(&mut self, numerator: u32, denominator: u32) -> bool;

    // === Protocol Output ===

    /// Add a message to the log
    pub fn add(&mut self, message_type: &str, args: &[&str]);

    /// Add a message with optional tags
    pub fn add_with_tags(&mut self, message_type: &str, args: &[&str], tags: &[(&str, &str)]);

    /// Get pending updates
    pub fn get_updates(&mut self) -> Vec<Update>;

    // === State Queries ===

    /// Check if battle is waiting for input
    pub fn needs_input(&self) -> bool;

    /// Check if battle can continue
    pub fn can_continue(&self) -> bool;
}

/// Battle creation options
#[derive(Debug, Clone)]
pub struct BattleOptions {
    pub format_id: FormatId,
    pub seed: Option<PrngSeed>,
    pub rated: bool,
    pub debug: bool,
}

/// Player setup options
#[derive(Debug, Clone)]
pub struct PlayerOptions {
    pub name: String,
    pub avatar: String,
    pub team: Option<Team>,
    pub rating: Option<u32>,
}
```

---

## Battle Actions (Move Execution)

```rust
/// Executes battle mechanics
pub struct BattleActions {
    /// Reference to parent battle (for queries)
    battle: *mut Battle,
}

impl BattleActions {
    // === Switch Mechanics ===

    /// Switch a Pokemon in
    pub fn switch_in(&mut self, pokemon: PokemonRef, pos: usize, is_drag: bool);

    /// Force a Pokemon out (Whirlwind, Dragon Tail)
    pub fn drag_in(&mut self, pokemon: PokemonRef, pos: usize);

    /// Run switch-in effects
    pub fn run_switch(&mut self, pokemon: PokemonRef);

    // === Move Mechanics ===

    /// Run a move (outer wrapper)
    pub fn run_move(
        &mut self,
        pokemon: PokemonRef,
        move_id: MoveId,
        target: Option<PokemonRef>,
        source_effect: Option<EffectId>,
        z_move: Option<MoveId>,
        max_move: Option<MoveId>,
        original_target: Option<PokemonRef>,
        external: bool,
    );

    /// Use a move (inner execution)
    pub fn use_move(
        &mut self,
        pokemon: PokemonRef,
        active_move: &ActiveMove,
        target: Option<PokemonRef>,
    ) -> bool;

    /// Try to use a move
    pub fn try_move_hit(
        &mut self,
        target: PokemonRef,
        pokemon: PokemonRef,
        active_move: &ActiveMove,
    ) -> HitResult;

    /// Move hits a target
    pub fn move_hit(
        &mut self,
        target: PokemonRef,
        pokemon: PokemonRef,
        active_move: &ActiveMove,
        is_secondary: bool,
    ) -> u32;  // Returns damage dealt

    // === Damage Mechanics ===

    /// Deal damage to a Pokemon
    pub fn damage(
        &mut self,
        pokemon: PokemonRef,
        damage: u32,
        source: Option<PokemonRef>,
        effect: Option<EffectId>,
    ) -> u32;  // Returns actual damage dealt

    /// Directly set HP
    pub fn set_hp(&mut self, pokemon: PokemonRef, hp: u32);

    /// Heal a Pokemon
    pub fn heal(
        &mut self,
        pokemon: PokemonRef,
        amount: u32,
        source: Option<PokemonRef>,
        effect: Option<EffectId>,
    ) -> u32;  // Returns actual healing

    /// Calculate damage
    pub fn calculate_damage(
        &self,
        pokemon: PokemonRef,
        target: PokemonRef,
        active_move: &ActiveMove,
        suppress_messages: bool,
    ) -> Option<DamageResult>;

    // === Status Mechanics ===

    /// Try to inflict a status condition
    pub fn try_set_status(
        &mut self,
        pokemon: PokemonRef,
        status: Status,
        source: Option<PokemonRef>,
        source_effect: Option<EffectId>,
    ) -> bool;

    /// Cure a status condition
    pub fn cure_status(&mut self, pokemon: PokemonRef, silent: bool);

    /// Try to add a volatile
    pub fn add_volatile(
        &mut self,
        pokemon: PokemonRef,
        volatile: Volatile,
        source: Option<PokemonRef>,
        source_effect: Option<EffectId>,
    ) -> bool;

    /// Remove a volatile
    pub fn remove_volatile(&mut self, pokemon: PokemonRef, volatile: Volatile);

    // === Stat Mechanics ===

    /// Boost stats
    pub fn boost(
        &mut self,
        pokemon: PokemonRef,
        boosts: &StatStages,
        source: Option<PokemonRef>,
        effect: Option<EffectId>,
    ) -> bool;

    /// Clear all boosts
    pub fn clear_boosts(&mut self, pokemon: PokemonRef);

    /// Copy boosts from one Pokemon to another
    pub fn copy_boosts(&mut self, source: PokemonRef, target: PokemonRef);

    // === Field Mechanics ===

    /// Set weather
    pub fn set_weather(
        &mut self,
        weather: Weather,
        source: Option<PokemonRef>,
        source_effect: Option<EffectId>,
    ) -> bool;

    /// Set terrain
    pub fn set_terrain(
        &mut self,
        terrain: Terrain,
        source: Option<PokemonRef>,
        source_effect: Option<EffectId>,
    ) -> bool;

    /// Add a side condition
    pub fn add_side_condition(
        &mut self,
        side: SideId,
        condition: SideCondition,
        source: Option<PokemonRef>,
        source_effect: Option<EffectId>,
    ) -> bool;

    // === Fainting ===

    /// Faint a Pokemon
    pub fn faint(&mut self, pokemon: PokemonRef, source: Option<PokemonRef>, effect: Option<EffectId>);

    /// Check and process faint queue
    pub fn faint_messages(&mut self);

    /// Check for battle end
    pub fn check_win(&mut self);

    // === Special Mechanics ===

    /// Mega evolve
    pub fn mega_evolve(&mut self, pokemon: PokemonRef);

    /// Dynamax
    pub fn dynamax(&mut self, pokemon: PokemonRef);

    /// Terastallize
    pub fn terastallize(&mut self, pokemon: PokemonRef);

    /// Transform (Ditto)
    pub fn transform(&mut self, pokemon: PokemonRef, target: PokemonRef);

    /// Run residual effects (end of turn)
    pub fn residual(&mut self);
}

/// Active move (move being used)
#[derive(Debug, Clone)]
pub struct ActiveMove {
    pub id: MoveId,
    pub base_move: &'static Move,

    /// Effective base power (after modifications)
    pub base_power: u32,

    /// Effective type
    pub move_type: Type,

    /// Effective category
    pub category: MoveCategory,

    /// Target type
    pub target: MoveTarget,

    /// Accuracy (None = always hits)
    pub accuracy: Option<u32>,

    /// Priority
    pub priority: i8,

    /// Flags
    pub flags: MoveFlags,

    /// Number of hits
    pub multi_hit: Option<MultiHit>,

    /// Secondary effects
    pub secondaries: Vec<Secondary>,

    /// Drain amount (1/2, etc.)
    pub drain: Option<(u32, u32)>,

    /// Recoil amount
    pub recoil: Option<(u32, u32)>,

    /// Z-move being used
    pub z_move: Option<MoveId>,

    /// Max move being used
    pub max_move: Option<MoveId>,

    /// Source effect (for called moves)
    pub source_effect: Option<EffectId>,

    /// Whether this is a spread move
    pub spread_hit: bool,

    /// Crit ratio stage
    pub crit_ratio: u8,

    /// Whether this is an external move (Dancer, etc.)
    pub external: bool,

    /// Total damage dealt
    pub total_damage: u32,

    /// Targets hit
    pub targets_hit: Vec<PokemonRef>,
}

#[derive(Debug, Clone)]
pub struct DamageResult {
    pub damage: u32,
    pub type_mod: f64,
    pub crit: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum HitResult {
    Hit,
    Miss,
    Immune,
    Blocked,
    Failed,
}
```

---

## Event System

The event system is the core of battle mechanics. Every ability, item, move effect, and status hooks into events.

```rust
/// Event identifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventId {
    // === Battle Flow ===
    Start,
    End,
    BeforeTurn,
    Residual,

    // === Move Flow ===
    BeforeMove,
    PrepareHit,
    ModifyMove,
    BasePower,
    ModifyDamage,
    TryMove,
    TryHit,
    TryImmunity,
    TryPrimaryHit,
    Accuracy,
    Hit,
    AfterHit,
    MoveAborted,
    AfterMoveSecondary,
    AfterMove,
    AfterSubDamage,

    // === Damage Flow ===
    Damage,
    ModifyAtk,
    ModifyDef,
    ModifySpA,
    ModifySpD,
    ModifySpe,
    ModifyAccuracy,
    ModifyPriority,
    ModifyCritRatio,
    CriticalHit,

    // === Switch Flow ===
    BeforeSwitchIn,
    BeforeSwitchOut,
    SwitchIn,
    SwitchOut,
    EntryHazard,
    RunSwitch,

    // === Status Flow ===
    SetStatus,
    TrySetStatus,
    CureStatus,
    AfterSetStatus,

    // === Volatile Flow ===
    TryAddVolatile,
    VolatileStart,
    VolatileEnd,

    // === Boost Flow ===
    TryBoost,
    Boost,
    AfterBoost,

    // === Field Flow ===
    WeatherStart,
    WeatherResidual,
    WeatherModifyDamage,
    TerrainStart,
    TerrainResidual,

    // === Type Flow ===
    ModifyType,
    ModifyTypePriority,
    Effectiveness,
    NegateImmunity,

    // === Item Flow ===
    TakeItem,
    UseItem,
    EatItem,

    // === Ability Flow ===
    SourceInvulnerabilityPriority,
    InvulnerabilityPriority,
    SourceModifyAccuracy,
    TargetModifyAccuracy,

    // === Targeting Flow ===
    RedirectTarget,
    ModifyTarget,

    // === Misc ===
    Faint,
    Update,
    DisableMove,
    LockMove,
    Attract,
    Trap,
    Immunity,
    StallMove,
    IsGrounded,
    NegateWeather,
    NegateTerrain,
}

/// A listener for an event
#[derive(Clone)]
pub struct EventListener {
    pub event_id: EventId,
    pub priority: i8,
    pub order: u8,
    pub handler: EventHandler,
}

/// Event handler function type
pub type EventHandler = fn(
    battle: &mut Battle,
    event: &Event,
) -> EventReturn;

/// Event data passed to handlers
pub struct Event {
    pub id: EventId,
    pub source: Option<PokemonRef>,
    pub target: Option<PokemonRef>,
    pub effect: Option<EffectId>,
    pub relay_var: Option<Box<dyn Any>>,
}

/// Event return type
#[derive(Debug, Clone)]
pub enum EventReturn {
    Continue,           // Continue processing
    Stop,               // Stop processing this event
    False,              // Return false (e.g., block the action)
    True,               // Return true
    Value(Box<dyn Any>),  // Return a value
    Damage(u32),        // Modified damage
    Priority(i8),       // Modified priority
}

impl Battle {
    /// Run an event through all applicable listeners
    pub fn run_event_internal(&mut self, event: Event) -> EventReturn {
        // 1. Collect all listeners for this event from:
        //    - Active Pokemon abilities
        //    - Active Pokemon items
        //    - Active move effects
        //    - Field conditions
        //    - Side conditions
        //    - Format rules

        // 2. Sort listeners by:
        //    - Order (lower first)
        //    - Priority (higher first)
        //    - Speed (higher first, unless Trick Room)

        // 3. Call each listener in order
        //    - Early return if a listener returns Stop/False

        // 4. Return final result
    }
}
```

### Effect Definitions

```rust
/// An effect (ability, item, move, condition)
#[derive(Debug, Clone)]
pub struct Effect {
    pub id: EffectId,
    pub name: String,
    pub effect_type: EffectType,
    pub listeners: Vec<EventListener>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectType {
    Ability,
    Item,
    Move,
    Status,
    Volatile,
    Weather,
    Terrain,
    SideCondition,
    Format,
}

/// Example: Ability with event handlers
///
/// Intimidate:
///   onSwitchIn: Lower opponent's Attack by 1 stage
///
/// pub fn intimidate() -> Effect {
///     Effect {
///         id: AbilityId::Intimidate,
///         name: "Intimidate".to_string(),
///         effect_type: EffectType::Ability,
///         listeners: vec![
///             EventListener {
///                 event_id: EventId::SwitchIn,
///                 priority: 0,
///                 order: 0,
///                 handler: |battle, event| {
///                     let pokemon = event.source.unwrap();
///                     for opponent in battle.get_adjacent_foes(pokemon) {
///                         battle.actions.boost(opponent, &StatStages { atk: -1, ..Default::default() }, Some(pokemon), Some(EffectId::Ability(AbilityId::Intimidate)));
///                     }
///                     EventReturn::Continue
///                 }
///             }
///         ]
///     }
/// }
```

---

## Dex (Data System)

The Dex provides data lookup for species, moves, abilities, items, etc.

```rust
/// The main Dex struct
pub struct Dex {
    /// Generation
    pub gen: u8,

    /// Format ID
    pub format_id: FormatId,

    /// Species data
    pub species: SpeciesIndex,

    /// Move data
    pub moves: MoveIndex,

    /// Ability data
    pub abilities: AbilityIndex,

    /// Item data
    pub items: ItemIndex,

    /// Condition data (statuses, volatiles)
    pub conditions: ConditionIndex,

    /// Type chart
    pub type_chart: TypeChart,

    /// Learnsets
    pub learnsets: LearnsetIndex,

    /// Natures
    pub natures: NatureIndex,
}

impl Dex {
    /// Load a Dex for a specific generation
    pub fn for_gen(gen: u8) -> Arc<Self>;

    /// Load a Dex for a specific format
    pub fn for_format(format_id: FormatId) -> Arc<Self>;

    /// Get type effectiveness
    pub fn get_effectiveness(&self, source: Type, target: &[Type]) -> f64;

    /// Check type immunity
    pub fn get_immunity(&self, source: Type, target: &[Type]) -> bool;
}

// === Species ===

#[derive(Debug, Clone)]
pub struct Species {
    pub id: SpeciesId,
    pub name: String,
    pub num: u16,

    /// Base stats
    pub base_stats: BaseStats,

    /// Types (1-2)
    pub types: Vec<Type>,

    /// Possible abilities
    pub abilities: SpeciesAbilities,

    /// Height in meters
    pub height_m: f32,

    /// Weight in kg
    pub weight_kg: f32,

    /// Gender ratio (male : female)
    pub gender_ratio: Option<(u8, u8)>,

    /// Egg groups
    pub egg_groups: Vec<EggGroup>,

    /// Base forme
    pub base_species: Option<SpeciesId>,

    /// Forme name
    pub forme: Option<String>,

    /// Other formes
    pub other_formes: Vec<SpeciesId>,

    /// Cosmetic formes
    pub cosmetic_formes: Vec<SpeciesId>,

    /// Evolution information
    pub evos: Vec<SpeciesId>,
    pub prevo: Option<SpeciesId>,

    /// Tags
    pub tags: Vec<SpeciesTag>,

    /// Can Gigantamax
    pub can_gigantamax: Option<MoveId>,

    /// Required item (for some formes)
    pub required_item: Option<ItemId>,

    /// Battle-only forme
    pub battle_only: Option<SpeciesId>,
}

#[derive(Debug, Clone)]
pub struct SpeciesAbilities {
    /// Primary ability
    pub primary: AbilityId,
    /// Secondary ability
    pub secondary: Option<AbilityId>,
    /// Hidden ability
    pub hidden: Option<AbilityId>,
    /// Special ability (form-specific)
    pub special: Option<AbilityId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpeciesTag {
    Sub,           // Sub-legendary
    RestrictedLegendary,
    Mythical,
    Paradox,       // Gen 9 Paradox Pokemon
}

// === Moves ===

#[derive(Debug, Clone)]
pub struct Move {
    pub id: MoveId,
    pub name: String,
    pub num: u16,

    /// Move type
    pub move_type: Type,

    /// Move category
    pub category: MoveCategory,

    /// Base power (0 for status moves)
    pub base_power: u32,

    /// Accuracy (None = always hits)
    pub accuracy: Option<u32>,

    /// PP (Power Points)
    pub pp: u32,

    /// Priority (-7 to +5)
    pub priority: i8,

    /// Targeting
    pub target: MoveTarget,

    /// Flags
    pub flags: MoveFlags,

    /// Multi-hit
    pub multi_hit: Option<MultiHit>,

    /// Secondary effects
    pub secondaries: Vec<Secondary>,

    /// Effect handlers
    pub effect: Effect,

    /// Z-move base power (if different)
    pub z_power: Option<u32>,

    /// Z-move effect (if not damage)
    pub z_effect: Option<ZMoveEffect>,

    /// Max move base power
    pub max_power: Option<u32>,

    /// Description
    pub desc: String,

    /// Short description
    pub short_desc: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveCategory {
    Physical,
    Special,
    Status,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveTarget {
    Normal,           // One adjacent Pokemon
    Self_,            // Self
    AdjacentAlly,     // One adjacent ally
    AdjacentAllyOrSelf,
    AdjacentFoe,      // One adjacent foe
    AllAdjacent,      // All adjacent Pokemon
    AllAdjacentFoes,  // All adjacent foes
    Allies,           // All allies (not self)
    AllySide,         // User's side of the field
    AllyTeam,         // User's team
    Any,              // Any Pokemon on the field
    FoeSide,          // Foe's side of the field
    RandomNormal,     // Random adjacent foe
    All,              // Entire field
    Scripted,         // Special (Counter, etc.)
}

#[derive(Debug, Clone, Default)]
pub struct MoveFlags {
    /// Makes contact
    pub contact: bool,
    /// Affected by Protect
    pub protect: bool,
    /// Can be reflected by Magic Coat
    pub reflectable: bool,
    /// Affected by Snatch
    pub snatch: bool,
    /// Can be copied by Mirror Move
    pub mirror: bool,
    /// Is a punch move (Iron Fist)
    pub punch: bool,
    /// Is a sound move (Soundproof)
    pub sound: bool,
    /// Is a powder move (Grass types immune)
    pub powder: bool,
    /// Is a bite move (Strong Jaw)
    pub bite: bool,
    /// Is a pulse move (Mega Launcher)
    pub pulse: bool,
    /// Is a bullet/ball move (Bulletproof)
    pub bullet: bool,
    /// Is a slicing move (Sharpness)
    pub slicing: bool,
    /// Is a wind move (Wind Power/Rider)
    pub wind: bool,
    /// Requires charging turn
    pub charge: bool,
    /// Requires recharge turn
    pub recharge: bool,
    /// Causes the user to faint
    pub selfdestruct: bool,
    /// Defrosting move
    pub defrost: bool,
    /// Can't be used twice in a row (Gorilla Tactics)
    pub cant_use_twice: bool,
    /// Heal move
    pub heal: bool,
    /// Hits non-adjacent in triples
    pub distance: bool,
    /// Prevents the user from moving if knocked out
    pub mental: bool,
    /// Hits through Substitute
    pub bypasssub: bool,
    /// Fails if target doesn't attack
    pub fails_no_target: bool,
    /// Fails if target is protected
    pub fails_protected: bool,
    /// Draining move
    pub drain: bool,
    /// Gravity fails this move
    pub gravity: bool,
    /// Dance move (Dancer)
    pub dance: bool,
}

#[derive(Debug, Clone)]
pub enum MultiHit {
    Fixed(u8),                // Always hits N times
    Range(u8, u8),            // Hits 2-5 times (or custom range)
    TwoToFive,                // Standard 2-5 (35-35-15-15 distribution)
    Calculated,               // Calculated at runtime (Triple Kick)
}

#[derive(Debug, Clone)]
pub struct Secondary {
    /// Chance (100 = always, unless ability changes it)
    pub chance: u8,
    /// Status to inflict
    pub status: Option<Status>,
    /// Volatile to inflict
    pub volatile: Option<Volatile>,
    /// Stat boosts/drops
    pub boosts: Option<StatStages>,
    /// Self effects
    pub is_self: bool,
}

// === Abilities ===

#[derive(Debug, Clone)]
pub struct Ability {
    pub id: AbilityId,
    pub name: String,
    pub num: u16,

    /// Effect handlers
    pub effect: Effect,

    /// Rating (for AI evaluation)
    pub rating: i8,

    /// Is suppressable
    pub is_breakable: bool,

    /// Description
    pub desc: String,

    /// Short description
    pub short_desc: String,
}

// === Items ===

#[derive(Debug, Clone)]
pub struct Item {
    pub id: ItemId,
    pub name: String,
    pub num: u16,

    /// Effect handlers
    pub effect: Effect,

    /// Fling power
    pub fling_power: Option<u32>,

    /// Is consumable
    pub is_gem: bool,
    pub is_berry: bool,

    /// Natural Gift type/power
    pub natural_gift_type: Option<Type>,
    pub natural_gift_power: Option<u32>,

    /// Mega stone
    pub mega_stone: Option<SpeciesId>,
    pub mega_evolves: Option<SpeciesId>,

    /// Z-crystal
    pub z_crystal: bool,
    pub z_move: Option<MoveId>,
    pub z_move_type: Option<Type>,
    pub z_move_from: Option<MoveId>,

    /// Is choice item
    pub is_choice: bool,

    /// Boost value
    pub boost_value: Option<f64>,

    /// Description
    pub desc: String,
}
```

---

## Protocol I/O (BattleStream)

```rust
/// Battle stream for I/O
pub struct BattleStream {
    /// The battle being simulated
    battle: Battle,

    /// Output buffer
    output: Vec<Update>,
}

/// An update to send to clients
#[derive(Debug, Clone)]
pub enum Update {
    /// Update for all players/spectators
    Broadcast(Vec<String>),

    /// Update for a specific player
    SideUpdate {
        side: Player,
        messages: Vec<String>,
    },

    /// Split update (different for player vs spectators)
    Split {
        side: Player,
        secret: Vec<String>,
        public: Vec<String>,
    },

    /// Battle has ended
    End(BattleEndData),
}

#[derive(Debug, Clone)]
pub struct BattleEndData {
    pub winner: Option<String>,
    pub turns: u32,
    pub seed: PrngSeed,
    pub log: Vec<String>,
}

impl BattleStream {
    /// Create a new battle stream
    pub fn new(dex: Arc<Dex>) -> Self;

    /// Write a command to the stream
    pub fn write(&mut self, input: &str) -> Result<(), StreamError>;

    /// Read pending updates
    pub fn read(&mut self) -> Vec<Update>;

    /// Process a command line
    fn process_line(&mut self, line: &str) -> Result<(), StreamError>;
}

impl BattleStream {
    fn process_line(&mut self, line: &str) -> Result<(), StreamError> {
        // Commands:
        // >start {"formatid":"gen9ou","seed":[...]}
        // >player p1 {"name":"Alice","team":"..."}
        // >player p2 {"name":"Bob","team":"..."}
        // >p1 move 1
        // >p2 switch 3
        // >forcewin p1
        // >tiebreak
        // >reseed seed

        let line = line.trim();
        if !line.starts_with('>') {
            return Ok(());  // Comment, ignore
        }

        let line = &line[1..];  // Remove >
        let (cmd, args) = line.split_once(' ').unwrap_or((line, ""));

        match cmd {
            "start" => self.cmd_start(args),
            "player" => self.cmd_player(args),
            "p1" | "p2" | "p3" | "p4" => self.cmd_choose(cmd, args),
            "forcewin" => self.cmd_forcewin(args),
            "tiebreak" => self.cmd_tiebreak(),
            "reseed" => self.cmd_reseed(args),
            _ => Ok(()),
        }
    }
}
```

---

## PRNG (Deterministic Randomness)

```rust
/// PRNG seed (4 numbers for Gen 5 RNG, or sodium seed)
#[derive(Debug, Clone)]
pub enum PrngSeed {
    /// Gen 5 style (4 x 16-bit numbers)
    Gen5([u16; 4]),
    /// Sodium style (32 bytes)
    Sodium([u8; 32]),
}

/// Pseudorandom number generator
pub struct Prng {
    seed: PrngSeed,
    state: PrngState,
}

enum PrngState {
    Gen5 {
        s0: u64,
        s1: u64,
        s2: u64,
        s3: u64,
    },
    Sodium {
        buffer: [u8; 64],
        index: usize,
        counter: u64,
    },
}

impl Prng {
    /// Create a new PRNG with a seed
    pub fn new(seed: PrngSeed) -> Self;

    /// Create with random seed
    pub fn random() -> Self;

    /// Get current seed
    pub fn get_seed(&self) -> &PrngSeed;

    /// Random float in [0, 1)
    pub fn random(&mut self) -> f64;

    /// Random integer in [0, n)
    pub fn random_int(&mut self, n: u32) -> u32;

    /// Random integer in [m, n)
    pub fn random_range(&mut self, m: u32, n: u32) -> u32;

    /// Check if random chance succeeds
    pub fn random_chance(&mut self, numerator: u32, denominator: u32) -> bool;

    /// Pick a random element from a slice
    pub fn sample<T>(&mut self, items: &[T]) -> Option<&T>;

    /// Pick and remove a random element
    pub fn sample_remove<T>(&mut self, items: &mut Vec<T>) -> Option<T>;

    /// Shuffle a slice in place (Fischer-Yates)
    pub fn shuffle<T>(&mut self, items: &mut [T]);
}

impl Prng {
    /// Gen 5 constants
    const GEN5_A: u64 = 0x5D588B656C078965;
    const GEN5_C: u64 = 0x00269EC3;

    fn gen5_next(&mut self) -> u32 {
        if let PrngState::Gen5 { ref mut s0, ref mut s1, ref mut s2, ref mut s3 } = self.state {
            // XOR the state parts
            let s = *s0 ^ *s1 ^ *s2 ^ *s3;
            // Advance each part with LCG
            *s0 = s0.wrapping_mul(Self::GEN5_A).wrapping_add(Self::GEN5_C);
            // Rotate state
            let tmp = *s3;
            *s3 = *s2;
            *s2 = *s1;
            *s1 = *s0;
            *s0 = tmp;
            // Return high 32 bits
            (s >> 32) as u32
        } else {
            unreachable!()
        }
    }
}
```

---

## Integration with kazam-client

The simulator can be used standalone or integrated with the kazam-client for bot development.

```rust
use kazam_client::{KazamClient, KazamHandler, KazamHandle};
use kazam_protocol::{BattleRequest, ServerMessage};
use kazam_simulator::{Battle, BattleStream, Dex};

/// Example: Using the simulator to predict battle outcomes
struct PredictiveBot {
    handle: KazamHandle,
    simulator: Option<BattleStream>,
    my_team: Option<String>,
    opponent_team: Vec<String>,  // Revealed Pokemon
}

impl KazamHandler for PredictiveBot {
    async fn on_request(&mut self, room_id: &str, request: &BattleRequest) {
        // Use simulator to evaluate moves
        if let Some(ref mut sim) = self.simulator {
            // Clone current state
            let state = sim.clone_state();

            // Try each possible move
            let mut best_move = 0;
            let mut best_score = i32::MIN;

            for (i, move_slot) in request.active[0].moves.iter().enumerate() {
                if move_slot.disabled || move_slot.pp == 0 {
                    continue;
                }

                // Simulate this move
                let mut test_sim = state.clone();
                test_sim.write(&format!(">p1 move {}", i + 1)).ok();
                test_sim.write(">p2 default").ok();  // Assume opponent does something

                // Evaluate resulting state
                let score = evaluate_state(&test_sim);
                if score > best_score {
                    best_score = score;
                    best_move = i;
                }
            }

            // Make the choice
            self.handle.choose(room_id, &format!("move {}", best_move + 1), request.rqid).ok();
        }
    }
}

fn evaluate_state(sim: &BattleStream) -> i32 {
    // Simple evaluation: HP differential
    let battle = sim.battle();
    let my_hp: u32 = battle.sides[0].pokemon.iter()
        .map(|p| p.hp)
        .sum();
    let opp_hp: u32 = battle.sides[1].pokemon.iter()
        .map(|p| p.hp)
        .sum();
    (my_hp as i32) - (opp_hp as i32)
}
```

### Building Battle State from Protocol Messages

The simulator output can be fed directly to kazam-client's battle state tracker:

```rust
use kazam_protocol::{parse_server_message, ServerMessage};
use kazam_battle::Battle as BattleState;

// Simulator emits protocol messages
let updates = simulator.read();
for update in updates {
    match update {
        Update::Broadcast(messages) => {
            for msg in messages {
                // Parse and update battle state
                if let Ok(parsed) = parse_server_message(&msg) {
                    battle_state.update(&parsed);
                }
            }
        }
        // Handle other update types...
    }
}
```

---

## Crate Structure

```
kazam-simulator/
├── Cargo.toml
├── src/
│   ├── lib.rs              # Public API
│   │
│   ├── battle/
│   │   ├── mod.rs          # Battle struct
│   │   ├── actions.rs      # BattleActions
│   │   ├── queue.rs        # BattleQueue
│   │   ├── stream.rs       # BattleStream I/O
│   │   └── events.rs       # Event system
│   │
│   ├── state/
│   │   ├── mod.rs
│   │   ├── pokemon.rs      # BattlePokemon
│   │   ├── side.rs         # BattleSide
│   │   ├── field.rs        # Field conditions
│   │   └── choice.rs       # Choice parsing
│   │
│   ├── types/
│   │   ├── mod.rs
│   │   ├── type_chart.rs   # Type effectiveness
│   │   ├── status.rs       # Status conditions
│   │   ├── stats.rs        # Stats and stages
│   │   └── nature.rs       # Natures
│   │
│   ├── dex/
│   │   ├── mod.rs          # Dex struct
│   │   ├── species.rs      # Species data
│   │   ├── moves.rs        # Move data
│   │   ├── abilities.rs    # Ability data
│   │   ├── items.rs        # Item data
│   │   └── conditions.rs   # Condition data
│   │
│   ├── prng.rs             # PRNG implementation
│   │
│   ├── team/
│   │   ├── mod.rs
│   │   ├── set.rs          # PokemonSet
│   │   └── pack.rs         # Team packing/unpacking
│   │
│   └── validate/
│       ├── mod.rs          # Team validation
│       ├── learnset.rs     # Move legality
│       └── rules.rs        # Format rules
│
├── data/
│   ├── gen9/
│   │   ├── species.json
│   │   ├── moves.json
│   │   ├── abilities.json
│   │   ├── items.json
│   │   └── learnsets.json
│   └── ...                 # Gen 1-8 data
│
└── tests/
    ├── battle_tests.rs     # Integration tests
    ├── event_tests.rs      # Event system tests
    ├── replay_tests.rs     # Replay compatibility
    └── protocol_tests.rs   # Protocol output tests
```

---

## Implementation Phases

### Phase 1: Core Framework

- [ ] Type system (Type, Effectiveness)
- [ ] Status conditions
- [ ] Stats and stat stages
- [ ] Natures
- [ ] PRNG implementation
- [ ] Basic data structures (Species, Move, Ability, Item)
- [ ] Data loading from JSON files

### Phase 2: Battle Infrastructure

- [ ] BattlePokemon state
- [ ] BattleSide state
- [ ] Field state
- [ ] Battle struct
- [ ] Choice parsing
- [ ] Action queue

### Phase 3: Event System

- [ ] Event definitions
- [ ] Event dispatcher
- [ ] Listener collection
- [ ] Priority sorting

### Phase 4: Core Mechanics

- [ ] Damage calculation
- [ ] Move execution
- [ ] Status infliction/curing
- [ ] Stat modification
- [ ] Switching

### Phase 5: Field Effects

- [ ] Weather
- [ ] Terrain
- [ ] Pseudo-weather
- [ ] Side conditions
- [ ] Entry hazards

### Phase 6: Protocol I/O

- [ ] BattleStream
- [ ] Input parsing
- [ ] Output formatting
- [ ] Split messages
- [ ] Request generation

### Phase 7: Gen 9 Mechanics

- [ ] All abilities
- [ ] All items
- [ ] All moves
- [ ] Terastallization

### Phase 8: Special Mechanics

- [ ] Mega Evolution
- [ ] Z-Moves
- [ ] Dynamax/Gigantamax
- [ ] Transform

### Phase 9: Team Validation

- [ ] Learnset validation
- [ ] Format rules
- [ ] Species clauses

### Phase 10: Earlier Generations

- [ ] Gen 8 differences
- [ ] Gen 7 differences
- [ ] Gen 1-6 differences

---

## Testing Strategy

### Protocol Compatibility

Compare output with Pokemon Showdown's simulator:

```rust
#[test]
fn test_protocol_compatibility() {
    // Run same battle in both simulators
    let seed = [1234, 5678, 9012, 3456];

    // Our simulator
    let mut ours = BattleStream::new(Dex::for_gen(9));
    ours.write(">start {\"formatid\":\"gen9ou\",\"seed\":[1234,5678,9012,3456]}").unwrap();

    // Compare outputs...
}
```

### Replay Testing

Parse and replay official replays:

```rust
#[test]
fn test_replay() {
    let replay = include_str!("replays/gen9ou-1234567.log");
    let mut stream = BattleStream::new(Dex::for_gen(9));

    for line in replay.lines() {
        if line.starts_with('>') {
            stream.write(line).unwrap();
        } else {
            // Verify output matches
            let output = stream.read();
            assert_eq!(output, expected);
        }
    }
}
```

### Damage Calculator Verification

```rust
#[test]
fn test_damage_calc() {
    // Compare with damage-calc library results
    let attacker = make_pokemon("Pikachu", 100);
    let defender = make_pokemon("Charizard", 100);
    let result = calculate_damage(&attacker, &defender, MoveId::ThunderBolt);

    // Should match expected range
    assert!(result.min >= 75);
    assert!(result.max <= 89);
}
```

---

## Performance Considerations

### Memory Layout

- Use indices instead of references where possible
- Pool allocations for common objects
- Avoid unnecessary cloning

### Speed Optimization

- Cache frequently-accessed data
- Use efficient data structures (arrays over hashmaps where appropriate)
- Minimize allocations in hot paths

### Parallel Simulation

For AI training, consider parallel battle simulation:

```rust
// Run many battles in parallel
let results: Vec<_> = (0..1000)
    .into_par_iter()
    .map(|_| {
        let mut battle = create_random_battle();
        run_to_completion(&mut battle)
    })
    .collect();
```

---

## External Dependencies

```toml
[dependencies]
# Kazam ecosystem
kazam-protocol = { path = "../protocol" }  # Wire format types
kazam-battle = { path = "../battle" }       # Domain types + state tracking

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Error handling
thiserror = "1.0"
anyhow = "1.0"

# PRNG (for sodium RNG)
rand = "0.8"
chacha20 = "0.9"  # For sodium-compatible RNG

# Parallelism (optional, for AI training)
rayon = { version = "1.8", optional = true }
```

---

## Notes on Pokemon Showdown Differences

While aiming for protocol compatibility, some implementation details may differ:

1. **Internal State**: Pokemon Showdown uses mutable object references extensively. Rust will use indices and explicit state management.

2. **Event System**: PS uses JavaScript callbacks. Rust will use function pointers or trait objects.

3. **Data Loading**: PS loads data lazily and has a mod system. We'll use compile-time generation where possible.

4. **String Handling**: PS uses string IDs extensively. We'll use enum IDs with string conversion methods.

5. **Memory Management**: PS relies on garbage collection. Rust will use explicit ownership.

The key requirement is that given the same seed and inputs, the output protocol messages should be byte-for-byte identical to Pokemon Showdown's output.
