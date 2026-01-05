/// Commands that clients can send to server
#[derive(Debug, Clone, PartialEq)]
pub enum ClientCommand {
    /// /trn USERNAME,0,ASSERTION
    TrustedLogin { username: String, assertion: String },

    /// /join ROOMID
    JoinRoom(String),

    /// /leave ROOMID
    LeaveRoom(String),

    /// /challenge USERNAME, FORMAT
    Challenge { username: String, format: String },

    /// /utm TEAM
    UpdateTeam(String),

    /// /search FORMAT
    Search(String),

    /// Raw chat message
    Chat(String),

    /// Raw command for catch-all
    Raw(String),
}

impl ClientCommand {
    /// Serialize command to protocol format
    pub fn to_protocol_string(&self) -> String {
        match self {
            Self::TrustedLogin {
                username,
                assertion,
            } => format!("/trn {},{}", username, assertion),
            Self::JoinRoom(room) => format!("/join {}", room),
            Self::LeaveRoom(room) => format!("/leave {}", room),
            Self::Challenge { username, format } => format!("/challenge {}, {}", username, format),
            Self::UpdateTeam(team) => format!("/utm {}", team),
            Self::Search(format) => format!("/search {}", format),
            Self::Chat(message) => message.clone(),
            Self::Raw(command) => command.clone(),
        }
    }
}

/// Client message with optional room context
pub struct ClientMessage {
    pub room_id: Option<String>,
    pub command: ClientCommand,
}

impl ClientMessage {
    /// Serialize to wire format: ROOMID|TEXT or |TEXT
    pub fn to_wire_format(&self) -> String {
        let text = self.command.to_protocol_string();
        match &self.room_id {
            Some(room) => format!("{}|{}", room, text),
            None => format!("|{}", text),
        }
    }
}
