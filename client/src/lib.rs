mod auth;
mod connection;
pub mod room;
mod state;

use std::collections::HashMap;

use anyhow::{Result, anyhow};
use connection::Connection;

pub use kazam_protocol::{ClientCommand, ClientMessage, ServerFrame, ServerMessage};

pub use room::{RoomId, RoomState, RoomType};
pub use state::UserInfo;

use state::ClientState;

/// Main Pokemon Showdown client
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

    /// Attempt to login with username and password
    ///
    /// This will fail if challstr has not been received yet.
    /// Check `can_login()` before calling, or wait for `ServerMessage::Challstr`.
    pub async fn login(&self, username: &str, password: &str) -> Result<()> {
        let challstr = self
            .state
            .challstr
            .as_ref()
            .ok_or_else(|| anyhow!("Cannot login: challstr not received yet"))?;

        let assertion = auth::get_assertion(username, password, challstr).await?;

        let cmd = ClientMessage {
            room_id: Some(String::new()),
            command: ClientCommand::TrustedLogin {
                username: username.to_string(),
                assertion,
            },
        };

        println!("{}", cmd.to_wire_format());
        self.connection.send(cmd.to_wire_format()).await
    }

    /// Get the next frame from the server
    pub async fn next_frame(&mut self) -> Option<Result<ServerFrame>> {
        let frame = self.connection.recv().await?;

        match frame {
            Ok(ref f) => {
                self.update_state(f);
                Some(Ok(f.clone()))
            }
            Err(e) => Some(Err(e)),
        }
    }

    /// Update internal state based on server messages
    fn update_state(&mut self, frame: &ServerFrame) {
        for msg in &frame.messages {
            match msg {
                ServerMessage::Challstr(challstr) => {
                    self.state.challstr = Some(challstr.clone());
                }
                ServerMessage::UpdateUser {
                    username,
                    named,
                    avatar,
                } => {
                    self.state.user = Some(UserInfo {
                        username: username.clone(),
                        logged_in: *named,
                        avatar: avatar.clone(),
                    });
                }
                _ => {}
            }
        }
    }

    /// Send a chat message to a room
    pub async fn send_chat(&self, room: &RoomId, message: &str) -> Result<()> {
        let cmd = ClientMessage {
            room_id: Some(room.0.clone()),
            command: ClientCommand::Chat(message.to_string()),
        };
        self.connection.send(cmd.to_wire_format()).await
    }

    /// Join a room
    pub async fn join_room(&self, room: &str) -> Result<()> {
        let cmd = ClientMessage {
            room_id: None,
            command: ClientCommand::JoinRoom(room.to_string()),
        };
        self.connection.send(cmd.to_wire_format()).await
    }

    /// Leave a room
    pub async fn leave_room(&self, room: &RoomId) -> Result<()> {
        let cmd = ClientMessage {
            room_id: None,
            command: ClientCommand::LeaveRoom(room.0.clone()),
        };
        self.connection.send(cmd.to_wire_format()).await
    }

    /// Check if login is possible (challstr has been received)
    pub fn can_login(&self) -> bool {
        self.state.challstr.is_some()
    }

    /// Get current user info
    pub fn user(&self) -> Option<&UserInfo> {
        self.state.user.as_ref()
    }

    /// Get all joined rooms
    pub fn rooms(&self) -> &HashMap<RoomId, RoomState> {
        &self.state.rooms
    }

    /// Get a specific room's state
    pub fn room(&self, id: &RoomId) -> Option<&RoomState> {
        self.state.rooms.get(id)
    }
}
