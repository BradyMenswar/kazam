use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};

use anyhow::{anyhow, Result};
use kazam_protocol::{BattleInfo, ClientCommand, ClientMessage};
use tokio::sync::mpsc;

use crate::room::RoomState;

const LOGIN_URL: &str = "https://play.pokemonshowdown.com/api/login";

pub struct ClientState {
    pub rooms: RwLock<HashMap<String, RoomState>>,
    pub battles: RwLock<HashMap<String, BattleInfo>>,
    pub logged_in: AtomicBool,
}

impl ClientState {
    pub fn new() -> Self {
        Self {
            rooms: RwLock::new(HashMap::new()),
            battles: RwLock::new(HashMap::new()),
            logged_in: AtomicBool::new(false),
        }
    }
}

#[derive(Clone)]
pub struct KazamHandle {
    tx: mpsc::UnboundedSender<ClientMessage>,
    state: Arc<ClientState>,
}

impl KazamHandle {
    pub fn new(tx: mpsc::UnboundedSender<ClientMessage>, state: Arc<ClientState>) -> Self {
        Self { tx, state }
    }

    fn send(&self, msg: ClientMessage) -> Result<()> {
        self.tx
            .send(msg)
            .map_err(|_| anyhow!("Client disconnected"))
    }

    pub async fn login(&self, username: &str, password: &str, challstr: &str) -> Result<()> {
        let assertion = get_assertion(username, password, challstr).await?;
        self.send(ClientMessage {
            room_id: Some(String::new()),
            command: ClientCommand::TrustedLogin {
                username: username.to_string(),
                assertion,
            },
        })
    }

    pub fn join_room(&self, room: &str) -> Result<()> {
        self.send(ClientMessage {
            room_id: None,
            command: ClientCommand::JoinRoom(room.to_string()),
        })
    }

    pub fn leave_room(&self, room: &str) -> Result<()> {
        self.send(ClientMessage {
            room_id: None,
            command: ClientCommand::LeaveRoom(room.to_string()),
        })
    }

    pub fn send_chat(&self, room: &str, message: &str) -> Result<()> {
        self.send(ClientMessage {
            room_id: Some(room.to_string()),
            command: ClientCommand::Chat(message.to_string()),
        })
    }

    pub fn send_raw(&self, message: &str) -> Result<()> {
        self.send(ClientMessage {
            room_id: None,
            command: ClientCommand::Raw(message.to_string()),
        })
    }

    pub fn search(&self, format: &str) -> Result<()> {
        self.send(ClientMessage {
            room_id: None,
            command: ClientCommand::Search(format.to_string()),
        })
    }

    pub fn cancel_search(&self) -> Result<()> {
        self.send(ClientMessage {
            room_id: None,
            command: ClientCommand::CancelSearch,
        })
    }

    pub fn choose(&self, room: &str, choice: &str, rqid: Option<u64>) -> Result<()> {
        self.send(ClientMessage {
            room_id: Some(room.to_string()),
            command: ClientCommand::Choose {
                choice: choice.to_string(),
                rqid,
            },
        })
    }

    pub fn forfeit(&self, room: &str) -> Result<()> {
        self.send(ClientMessage {
            room_id: Some(room.to_string()),
            command: ClientCommand::Forfeit,
        })
    }

    pub fn timer(&self, room: &str, on: bool) -> Result<()> {
        self.send(ClientMessage {
            room_id: Some(room.to_string()),
            command: ClientCommand::Timer(on),
        })
    }

    pub fn is_logged_in(&self) -> bool {
        self.state.logged_in.load(Ordering::Relaxed)
    }

    pub fn get_room(&self, room_id: &str) -> Option<RoomState> {
        self.state.rooms.read().ok()?.get(room_id).cloned()
    }

    pub fn rooms(&self) -> Vec<String> {
        self.state
            .rooms
            .read()
            .map(|r| r.keys().cloned().collect())
            .unwrap_or_default()
    }

    pub fn in_room(&self, room_id: &str) -> bool {
        self.state
            .rooms
            .read()
            .map(|r| r.contains_key(room_id))
            .unwrap_or(false)
    }

    pub fn get_battle(&self, room_id: &str) -> Option<BattleInfo> {
        self.state.battles.read().ok()?.get(room_id).cloned()
    }

    pub fn in_battle(&self, room_id: &str) -> bool {
        self.state
            .battles
            .read()
            .map(|b| b.contains_key(room_id))
            .unwrap_or(false)
    }
}

async fn get_assertion(username: &str, password: &str, challstr: &str) -> Result<String> {
    let client = reqwest::Client::new();

    let params = [
        ("name", username),
        ("pass", password),
        ("challstr", challstr),
    ];

    let response = client.post(LOGIN_URL).form(&params).send().await?;
    let text = response.text().await?;

    // Response is prefixed with "]"
    let json_str = text.trim_start_matches(']');
    let json: serde_json::Value = serde_json::from_str(json_str)?;

    if let Some(assertion) = json.get("assertion").and_then(|v| v.as_str()) {
        if let Some(error_msg) = assertion.strip_prefix(";;") {
            return Err(anyhow!("Login failed: {}", error_msg));
        }
        Ok(assertion.to_string())
    } else {
        Err(anyhow!("Login response missing assertion"))
    }
}
