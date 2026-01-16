//! Battle state tracking from server messages

mod battle;
mod updater;

pub use battle::{player_to_index, position_to_slot, TrackedBattle};
