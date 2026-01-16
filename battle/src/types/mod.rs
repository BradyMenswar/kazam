//! Domain types for battle state tracking

mod conditions;
mod field;
mod pokemon;
mod pokemon_type;
mod side;
mod stats;
mod status;

pub use conditions::{SideCondition, SideConditionState, Terrain, Weather};
pub use field::FieldState;
pub use pokemon::{PokemonIdentity, PokemonState};
pub use pokemon_type::{Type, TYPE_CHART};
pub use side::SideState;
pub use stats::StatStages;
pub use status::{Status, Volatile};
