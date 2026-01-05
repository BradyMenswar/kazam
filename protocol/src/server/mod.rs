mod tests;

use crate::ParseError;
use anyhow::Result;

#[derive(Debug, Clone, PartialEq)]
pub enum ServerMessage {
    /// |challstr|CHALLSTR
    Challstr(String),

    /// |updateuser|USER|NAMED|AVATAR|SETTINGS
    UpdateUser {
        username: String,
        named: bool,
        avatar: String,
    },

    /// |nametaken|USERNAME|MESSAGE
    NameTaken { username: String, message: String },

    /// Raw message for catch-all
    Raw(String),
}

/// Wrapper for multiline-capable server messages
#[derive(Debug, Clone, PartialEq)]
pub struct ServerFrame {
    pub room_id: Option<String>,
    pub messages: Vec<ServerMessage>,
}

/// Parse a complete WebSocket frame into structured messages
pub fn parse_server_frame(frame: &str) -> Result<ServerFrame> {
    let mut lines = frame.lines();
    let mut room_id = None;

    // Check if first line is >ROOMID
    if let Some(first_line) = lines.clone().next() {
        if let Some(room) = first_line.strip_prefix('>') {
            room_id = Some(room.to_string());
            lines.next();
        }
    }

    // Parse remaining lines as messages
    let messages: Vec<ServerMessage> = lines
        .filter(|line| !line.trim().is_empty())
        .map(parse_server_message)
        .collect::<anyhow::Result<Vec<_>>>()?;

    Ok(ServerFrame { room_id, messages })
}

/// Parse a single line from the server into a ServerMessage
pub fn parse_server_message(line: &str) -> Result<ServerMessage> {
    let line = line.trim();

    if line.is_empty() {
        return Ok(ServerMessage::Raw(String::new()));
    }

    if !line.starts_with('|') {
        return Ok(ServerMessage::Raw(line.to_string()));
    }

    let parts: Vec<&str> = line.split('|').collect();

    if parts.len() < 2 {
        return Ok(ServerMessage::Raw(line.to_string()));
    }

    match parts[1] {
        "challstr" => parse_challstr(&parts),
        "updateuser" => parse_updateuser(&parts),
        "nametaken" => parse_nametaken(&parts),
        _ => Ok(ServerMessage::Raw(line.to_string())),
    }
}

fn parse_challstr(parts: &[&str]) -> Result<ServerMessage> {
    if parts.len() < 3 {
        return Err(ParseError::MissingField("challstr value".to_string()).into());
    }

    // CHALLSTR can contain | characters, so join everything after parts[1]
    let challstr = parts[2..].join("|");
    if challstr.is_empty() {
        return Err(ParseError::InvalidFormat("challstr cannot be empty".to_string()).into());
    }

    Ok(ServerMessage::Challstr(challstr))
}

fn parse_updateuser(parts: &[&str]) -> Result<ServerMessage> {
    if parts.len() < 4 {
        return Err(ParseError::MissingField("updateuser fields".to_string()).into());
    }

    let user_str = parts[2];
    let username = user_str.trim_start_matches(|c: char| !c.is_alphanumeric());

    let named = parts[3] == "1";
    let avatar = parts.get(4).unwrap_or(&"").to_string();

    Ok(ServerMessage::UpdateUser {
        username: username.to_string(),
        named,
        avatar,
    })
}

fn parse_nametaken(parts: &[&str]) -> Result<ServerMessage> {
    if parts.len() < 4 {
        return Err(ParseError::MissingField("nametaken fields".to_string()).into());
    }

    Ok(ServerMessage::NameTaken {
        username: parts[2].to_string(),
        message: parts[3..].join("|"),
    })
}
