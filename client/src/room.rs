use kazam_protocol::{RoomType, User};

#[derive(Debug, Clone)]
pub struct RoomState {
    pub id: String,
    pub room_type: RoomType,
    pub title: Option<String>,
    pub users: Vec<User>,
}
