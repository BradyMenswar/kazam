//! Query helpers for battle decision making
//!
//! This module provides utilities for analyzing type matchups and
//! other battle queries useful for bot decision making.

mod matchup;

pub use matchup::{
    // Type-level queries
    immunities,
    is_immune_to,
    is_weak_to_any,
    resistances,
    resists_all,
    weaknesses,
};
