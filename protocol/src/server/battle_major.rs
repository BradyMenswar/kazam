//! Major battle action message parsers
//!
//! These are the primary actions in battle: moves, switches, faints, etc.

use super::battle::{parse_details, parse_hp_status, parse_pokemon, Pokemon};
use super::ServerMessage;
use anyhow::Result;

/// Parse |move|POKEMON|MOVE|TARGET with optional tags
pub fn parse_move(parts: &[&str]) -> Result<ServerMessage> {
    let pokemon = parse_pokemon(parts, 2)?;
    let move_name = parts.get(3).unwrap_or(&"").to_string();
    let target = parts.get(4).and_then(|s| Pokemon::parse(s));

    // Check for optional tags in remaining parts
    let mut miss = false;
    let mut still = false;
    let mut anim = None;

    for part in parts.iter().skip(5) {
        if *part == "[miss]" {
            miss = true;
        } else if *part == "[still]" {
            still = true;
        } else if let Some(anim_move) = part.strip_prefix("[anim] ") {
            anim = Some(anim_move.to_string());
        }
    }

    Ok(ServerMessage::Move {
        pokemon,
        move_name,
        target,
        miss,
        still,
        anim,
    })
}

/// Parse |switch|POKEMON|DETAILS|HP STATUS
pub fn parse_switch(parts: &[&str]) -> Result<ServerMessage> {
    let pokemon = parse_pokemon(parts, 2)?;
    let details = parse_details(parts, 3);
    let hp_status = parse_hp_status(parts, 4);

    Ok(ServerMessage::Switch {
        pokemon,
        details,
        hp_status,
    })
}

/// Parse |drag|POKEMON|DETAILS|HP STATUS
pub fn parse_drag(parts: &[&str]) -> Result<ServerMessage> {
    let pokemon = parse_pokemon(parts, 2)?;
    let details = parse_details(parts, 3);
    let hp_status = parse_hp_status(parts, 4);

    Ok(ServerMessage::Drag {
        pokemon,
        details,
        hp_status,
    })
}

/// Parse |detailschange|POKEMON|DETAILS|HP STATUS
pub fn parse_detailschange(parts: &[&str]) -> Result<ServerMessage> {
    let pokemon = parse_pokemon(parts, 2)?;
    let details = parse_details(parts, 3);
    let hp_status = parse_hp_status(parts, 4);

    Ok(ServerMessage::DetailsChange {
        pokemon,
        details,
        hp_status,
    })
}

/// Parse |-formechange|POKEMON|SPECIES|HP STATUS
pub fn parse_formechange(parts: &[&str]) -> Result<ServerMessage> {
    let pokemon = parse_pokemon(parts, 2)?;
    let species = parts.get(3).unwrap_or(&"").to_string();
    let hp_status = parse_hp_status(parts, 4);

    Ok(ServerMessage::FormeChange {
        pokemon,
        species,
        hp_status,
    })
}

/// Parse |replace|POKEMON|DETAILS|HP STATUS
pub fn parse_replace(parts: &[&str]) -> Result<ServerMessage> {
    let pokemon = parse_pokemon(parts, 2)?;
    let details = parse_details(parts, 3);
    let hp_status = parse_hp_status(parts, 4);

    Ok(ServerMessage::Replace {
        pokemon,
        details,
        hp_status,
    })
}

/// Parse |swap|POKEMON|POSITION
pub fn parse_swap(parts: &[&str]) -> Result<ServerMessage> {
    let pokemon = parse_pokemon(parts, 2)?;
    let position = parts
        .get(3)
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| anyhow::anyhow!("Missing position"))?;

    Ok(ServerMessage::Swap { pokemon, position })
}

/// Parse |cant|POKEMON|REASON or |cant|POKEMON|REASON|MOVE
pub fn parse_cant(parts: &[&str]) -> Result<ServerMessage> {
    let pokemon = parse_pokemon(parts, 2)?;
    let reason = parts.get(3).unwrap_or(&"").to_string();
    let move_name = parts.get(4).map(|s| s.to_string());

    Ok(ServerMessage::Cant {
        pokemon,
        reason,
        move_name,
    })
}

/// Parse |faint|POKEMON
pub fn parse_faint(parts: &[&str]) -> Result<ServerMessage> {
    let pokemon = parse_pokemon(parts, 2)?;
    Ok(ServerMessage::Faint(pokemon))
}
