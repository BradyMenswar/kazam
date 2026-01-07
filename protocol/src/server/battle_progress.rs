//! Battle progress message parsers
//!
//! These messages track the flow and state of a battle.

use super::ServerMessage;
use anyhow::Result;
use serde_json::Value;

/// Parse |request|REQUEST (JSON)
pub fn parse_request(parts: &[&str]) -> Result<ServerMessage> {
    let json_str = parts.get(2).unwrap_or(&"{}");
    let request: Value = serde_json::from_str(json_str)?;
    Ok(ServerMessage::Request(request))
}

/// Parse |inactive|MESSAGE
pub fn parse_inactive(parts: &[&str]) -> Result<ServerMessage> {
    let message = parts.get(2).unwrap_or(&"").to_string();
    Ok(ServerMessage::Inactive(message))
}

/// Parse |inactiveoff|MESSAGE
pub fn parse_inactiveoff(parts: &[&str]) -> Result<ServerMessage> {
    let message = parts.get(2).unwrap_or(&"").to_string();
    Ok(ServerMessage::InactiveOff(message))
}

/// Parse |upkeep
pub fn parse_upkeep(_parts: &[&str]) -> Result<ServerMessage> {
    Ok(ServerMessage::Upkeep)
}

/// Parse |turn|NUMBER
pub fn parse_turn(parts: &[&str]) -> Result<ServerMessage> {
    let turn = parts
        .get(2)
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| anyhow::anyhow!("Missing turn number"))?;

    Ok(ServerMessage::Turn(turn))
}

/// Parse |win|USER
pub fn parse_win(parts: &[&str]) -> Result<ServerMessage> {
    let user = parts.get(2).unwrap_or(&"").to_string();
    Ok(ServerMessage::Win(user))
}

/// Parse |tie
pub fn parse_tie(_parts: &[&str]) -> Result<ServerMessage> {
    Ok(ServerMessage::Tie)
}
