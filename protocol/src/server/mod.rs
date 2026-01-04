mod tests;

use crate::ParseError;
use anyhow::Result;

#[derive(Debug, Clone, PartialEq)]
pub enum ServerMessage {
    Challstr(String),
    Raw(String),
}

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
        _ => Ok(ServerMessage::Raw(line.to_string())),
    }
}

fn parse_challstr(parts: &[&str]) -> Result<ServerMessage> {
    // |challstr|CHALLSTR
    // CHALLSTR can contain | characters, so join everything after parts[1]
    if parts.len() < 3 {
        return Err(ParseError::MissingField("challstr value".to_string()).into());
    }

    let challstr = parts[2..].join("|");
    if challstr.is_empty() {
        return Err(ParseError::InvalidFormat("challstr cannot be empty".to_string()).into());
    }

    Ok(ServerMessage::Challstr(challstr))
}
