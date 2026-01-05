use thiserror::Error;

pub mod client;
pub mod server;

pub use client::{ClientCommand, ClientMessage};
pub use server::{ServerFrame, ServerMessage, parse_server_frame, parse_server_message};

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Invalid message format: {0}")]
    InvalidFormat(String),

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Empty message")]
    EmptyMessage,
}
