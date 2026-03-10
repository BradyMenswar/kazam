//! Battle state tracking and domain types for Pokemon Showdown.
//!
//! This crate provides the shared type system used by both state tracking (for bots)
//! and simulation (for prediction/training).
//!
//! # Overview
//!
//! `kazam-battle` sits between `kazam-protocol` (wire format) and higher-level components:
//!
//! ```text
//! kazam-protocol (wire format)
//!        │
//!        ▼
//! kazam-battle (domain types + canonical state reduction) ← THIS CRATE
//!        │
//!        ├─> kazam-client (bots using tracked state)
//!        ├─> kazam-replay (timeline navigation over reduced state)
//!        └─> kazam-simulator (full simulation)
//! ```
//!
//! # Main Types
//!
//! ## Domain Types
//! - [`Type`] - Pokemon types with effectiveness chart
//! - [`Status`] - Non-volatile status conditions (Burn, Freeze, etc.)
//! - [`Volatile`] - Volatile conditions (Confusion, Taunt, etc.)
//! - [`StatStages`] - Stat stage modifiers (-6 to +6)
//! - [`Weather`], [`Terrain`], [`SideCondition`] - Field conditions
//! - [`PokemonState`] - Full Pokemon battle state
//! - [`SideState`] - One player's side of the battle
//! - [`FieldState`] - Global field conditions
//!
//! ## State Tracking
//! - [`TrackedBattle`] - Main entry point for reducing protocol messages into battle state
//! - [`BattleKnowledge`] - Declares whether the state is public-only, player-enriched, or omniscient
//! - [`BattleSnapshot`] / [`TurnSnapshot`] - Snapshot helpers for restoring or indexing reduced state
//!
//! # Example Usage
//!
//! ```ignore
//! use kazam_battle::{TrackedBattle, Weather, SideCondition};
//! use kazam_protocol::ServerMessage;
//!
//! let mut battle = TrackedBattle::new();
//!
//! // Process server messages
//! battle.update(&message);
//!
//! // Query battle state
//! if let Some(me) = battle.me() {
//!     let active = me.active_pokemon().unwrap();
//!     println!("My active: {} at {}%", active.name(), active.hp_percent());
//! }
//!
//! // Check field conditions
//! if battle.field.weather == Some(Weather::Sun) {
//!     println!("Sun is active!");
//! }
//! ```

pub mod query;
pub mod tracking;
pub mod types;

// Re-export main types at crate root for convenience
pub use tracking::{
    BattleKnowledge,
    BattleSnapshot,
    TrackedBattle,
    TurnSnapshot,
    player_to_index,
    position_to_slot,
};
pub use types::{
    FieldState, PokemonIdentity, PokemonState, SideCondition, SideConditionState, SideState,
    StatStages, Status, Terrain, Type, Volatile, Weather, TYPE_CHART,
};

// Re-export commonly used protocol types
pub use kazam_protocol::{GameType, Player, Stat};
