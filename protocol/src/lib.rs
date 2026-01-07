use thiserror::Error;

pub mod client;
pub mod server;

pub use client::{ClientCommand, ClientMessage};
pub use server::{
    ActivePokemon, BattleInfo, BattleRequest, ChallengeInfo, ChallengeState, Format, FormatSection,
    GameType, HpStatus, MaxMoveSlot, MaxMoves, MoveSlot, Player, PlayerInfo, Pokemon,
    PokemonDetails, PokemonStats, PreviewPokemon, RoomType, SearchState, ServerFrame,
    ServerMessage, Side, SideInfo, SidePokemon, Stat, User, ZMoveInfo, parse_server_frame,
    parse_server_message,
};

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Invalid message format: {0}")]
    InvalidFormat(String),

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Empty message")]
    EmptyMessage,
}
