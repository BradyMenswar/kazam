use std::hash::Hash;

/// Unique identifier for a room
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

/// Type of room
#[derive(Debug, Clone)]
pub enum RoomType {
    Chat,
    Battle { format: String },
}

/// State of a joined room
#[derive(Debug, Clone)]
pub struct RoomState {
    pub id: RoomId,
    pub room_type: RoomType,
    pub users: Vec<String>,
}
