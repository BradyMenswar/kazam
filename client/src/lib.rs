mod auth;
mod connection;
mod handler;
mod receiver;
pub mod room;
mod sender;
mod state;

use anyhow::Result;
use connection::Connection;

// Re-export protocol types
pub use kazam_protocol::{ClientCommand, ClientMessage, ServerFrame, ServerMessage};

// Re-export client types
pub use handler::Handler;
pub use receiver::Receiver;
pub use room::{RoomId, RoomState, RoomType};
pub use sender::Sender;
pub use state::UserInfo;

// Re-export async_trait for users implementing Handler
pub use async_trait::async_trait;

use state::ClientState;

/// Main Pokemon Showdown client
///
/// Use `connect()` to establish a connection, then `split()` to get
/// a `Receiver` and `Sender` for handling messages.
pub struct Client {
    connection: Connection,
    state: ClientState,
}

impl Client {
    /// Connect to a Pokemon Showdown server
    pub async fn connect(url: &str) -> Result<Self> {
        let connection = Connection::connect(url).await?;
        Ok(Self {
            connection,
            state: ClientState::new(),
        })
    }

    /// Split the client into a receiver and sender.
    ///
    /// The sender can be cloned and passed to handlers.
    /// The receiver drives the message loop.
    pub fn split(self) -> (Receiver, Sender) {
        let (incoming, outgoing) = self.connection.split();
        let receiver = Receiver::new(incoming, self.state);
        let sender = Sender::new(outgoing);
        (receiver, sender)
    }
}
