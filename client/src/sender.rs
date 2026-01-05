use anyhow::Result;
use tokio::sync::mpsc;

use crate::auth;
use crate::room::RoomId;
use kazam_protocol::{ClientCommand, ClientMessage};

/// Cloneable handle for sending messages to the server.
///
/// This can be passed to handlers and cloned freely.
#[derive(Clone)]
pub struct Sender {
    outgoing: mpsc::Sender<String>,
}

impl Sender {
    pub(crate) fn new(outgoing: mpsc::Sender<String>) -> Self {
        Self { outgoing }
    }

    /// Send a raw string to the server
    pub async fn send_raw(&self, message: String) -> Result<()> {
        self.outgoing
            .send(message)
            .await
            .map_err(|_| anyhow::anyhow!("Connection closed"))
    }

    /// Login with username and password using a stored challstr
    pub async fn login(&self, username: &str, password: &str, challstr: &str) -> Result<()> {
        let assertion = auth::get_assertion(username, password, challstr).await?;

        let cmd = ClientMessage {
            room_id: Some(String::new()),
            command: ClientCommand::TrustedLogin {
                username: username.to_string(),
                assertion,
            },
        };

        self.send_raw(cmd.to_wire_format()).await
    }

    /// Send a chat message to a room
    pub async fn send_chat(&self, room: &RoomId, message: &str) -> Result<()> {
        let cmd = ClientMessage {
            room_id: Some(room.0.clone()),
            command: ClientCommand::Chat(message.to_string()),
        };
        self.send_raw(cmd.to_wire_format()).await
    }

    /// Join a room
    pub async fn join_room(&self, room: &str) -> Result<()> {
        let cmd = ClientMessage {
            room_id: None,
            command: ClientCommand::JoinRoom(room.to_string()),
        };
        self.send_raw(cmd.to_wire_format()).await
    }

    /// Leave a room
    pub async fn leave_room(&self, room: &RoomId) -> Result<()> {
        let cmd = ClientMessage {
            room_id: None,
            command: ClientCommand::LeaveRoom(room.0.clone()),
        };
        self.send_raw(cmd.to_wire_format()).await
    }
}
