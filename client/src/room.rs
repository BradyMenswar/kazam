use anyhow::Result;
use kazam_protocol::{ClientCommand, ClientMessage};
use std::hash::Hash;

use crate::KazamClient;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RoomId(pub String);

impl RoomId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for RoomId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for RoomId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

#[derive(Debug, Clone)]
pub enum RoomType {
    Chat,
    Battle { format: String },
}

#[derive(Debug, Clone)]
pub struct RoomState {
    pub id: RoomId,
    pub room_type: RoomType,
    pub users: Vec<String>,
}

impl KazamClient {
    pub fn in_room(&self, room_id: &RoomId) -> bool {
        self.rooms.contains_key(room_id)
    }

    pub fn rooms(&self) -> impl Iterator<Item = &str> {
        self.rooms.iter().map(|s| s.0.as_str())
    }

    pub async fn join_room(&mut self, room: &str) -> Result<()> {
        let cmd = ClientMessage {
            room_id: None,
            command: ClientCommand::JoinRoom(room.to_string()),
        };
        self.send_raw(cmd.to_wire_format()).await
    }

    pub async fn leave_room(&mut self, room: &RoomId) -> Result<()> {
        let cmd = ClientMessage {
            room_id: None,
            command: ClientCommand::LeaveRoom(room.0.clone()),
        };
        self.send_raw(cmd.to_wire_format()).await
    }
}
