//! Replay loading, indexing, and playback controls for Pokemon Showdown logs.
//!
//! `kazam-replay` builds on top of `kazam-protocol` for parsing and
//! `kazam-battle` for canonical state reduction and snapshots.

mod controller;
mod error;
mod log;

pub use controller::{ReplayController, ReplaySpeed};
pub use error::ReplayError;
pub use log::{ReplayEvent, ReplayLog};
