//! Pokemon Showdown team formats and conversion utilities.
//!
//! `kazam-team` provides a canonical Rust model for teams plus codecs for:
//! - export format
//! - JSON format
//! - packed format

mod codec;
mod error;
mod model;

pub use codec::Teams;
pub use error::TeamError;
pub use model::{
    PokemonSet, StatLine, Team, default_dynamax_level, default_happiness, default_ivs,
    default_level,
};
