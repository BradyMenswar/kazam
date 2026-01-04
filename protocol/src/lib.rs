use thiserror::Error;

pub mod client;
pub mod server;

pub use server::{ServerMessage, parse_server_message};

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Invalid message format: {0}")]
    InvalidFormat(String),

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Empty message")]
    EmptyMessage,
}
