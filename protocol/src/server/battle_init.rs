//! Battle initialization message parsers
//!
//! These messages are sent at the start of a battle to set up the game state.

use super::battle::{GameType, Player, PokemonDetails};
use super::ServerMessage;
use anyhow::Result;

/// Parse |player|PLAYER|USERNAME|AVATAR|RATING
pub fn parse_player(parts: &[&str]) -> Result<ServerMessage> {
    let player = parts
        .get(2)
        .and_then(|s| Player::parse(s))
        .ok_or_else(|| anyhow::anyhow!("Missing player"))?;

    let username = parts.get(3).unwrap_or(&"").to_string();
    let avatar = parts.get(4).unwrap_or(&"").to_string();
    let rating = parts.get(5).and_then(|s| s.parse().ok());

    Ok(ServerMessage::BattlePlayer {
        player,
        username,
        avatar,
        rating,
    })
}

/// Parse |teamsize|PLAYER|NUMBER
pub fn parse_teamsize(parts: &[&str]) -> Result<ServerMessage> {
    let player = parts
        .get(2)
        .and_then(|s| Player::parse(s))
        .ok_or_else(|| anyhow::anyhow!("Missing player"))?;

    let size = parts
        .get(3)
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| anyhow::anyhow!("Missing team size"))?;

    Ok(ServerMessage::TeamSize { player, size })
}

/// Parse |gametype|GAMETYPE
pub fn parse_gametype(parts: &[&str]) -> Result<ServerMessage> {
    let game_type = parts
        .get(2)
        .and_then(|s| GameType::parse(s))
        .ok_or_else(|| anyhow::anyhow!("Missing game type"))?;

    Ok(ServerMessage::GameType(game_type))
}

/// Parse |gen|GENNUM
pub fn parse_gen(parts: &[&str]) -> Result<ServerMessage> {
    let generation = parts
        .get(2)
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| anyhow::anyhow!("Missing generation"))?;

    Ok(ServerMessage::Gen(generation))
}

/// Parse |tier|FORMATNAME
pub fn parse_tier(parts: &[&str]) -> Result<ServerMessage> {
    let format = parts.get(2).unwrap_or(&"").to_string();
    Ok(ServerMessage::Tier(format))
}

/// Parse |rated| or |rated|MESSAGE
pub fn parse_rated(parts: &[&str]) -> Result<ServerMessage> {
    let message = parts.get(2).map(|s| s.to_string());
    Ok(ServerMessage::Rated(message))
}

/// Parse |rule|RULE: DESCRIPTION
pub fn parse_rule(parts: &[&str]) -> Result<ServerMessage> {
    let rule = parts.get(2).unwrap_or(&"").to_string();
    Ok(ServerMessage::Rule(rule))
}

/// Parse |clearpoke
pub fn parse_clearpoke(_parts: &[&str]) -> Result<ServerMessage> {
    Ok(ServerMessage::ClearPoke)
}

/// Parse |poke|PLAYER|DETAILS|ITEM
pub fn parse_poke(parts: &[&str]) -> Result<ServerMessage> {
    let player = parts
        .get(2)
        .and_then(|s| Player::parse(s))
        .ok_or_else(|| anyhow::anyhow!("Missing player"))?;

    let details = parts
        .get(3)
        .map(|s| PokemonDetails::parse(s))
        .unwrap_or_default();

    let has_item = parts.get(4).map(|s| *s == "item").unwrap_or(false);

    Ok(ServerMessage::Poke {
        player,
        details,
        has_item,
    })
}

/// Parse |teampreview or |teampreview|NUMBER
pub fn parse_teampreview(parts: &[&str]) -> Result<ServerMessage> {
    let count = parts.get(2).and_then(|s| s.parse().ok());
    Ok(ServerMessage::TeamPreview(count))
}

/// Parse |start
pub fn parse_start(_parts: &[&str]) -> Result<ServerMessage> {
    Ok(ServerMessage::BattleStart)
}
