//! Minor battle action message parsers
//!
//! These are secondary effects in battle: damage, stat changes, status, etc.
//! In the official client, they're usually displayed in smaller font.

use super::battle::{parse_hp_status, parse_pokemon, Pokemon, Side, Stat};
use super::ServerMessage;
use anyhow::Result;

/// Parse |-fail|POKEMON|ACTION
pub fn parse_fail(parts: &[&str]) -> Result<ServerMessage> {
    let pokemon = parse_pokemon(parts, 2)?;
    let action = parts.get(3).map(|s| s.to_string());

    Ok(ServerMessage::Fail { pokemon, action })
}

/// Parse |-block|POKEMON|EFFECT|MOVE|ATTACKER
pub fn parse_block(parts: &[&str]) -> Result<ServerMessage> {
    let pokemon = parse_pokemon(parts, 2)?;
    let effect = parts.get(3).unwrap_or(&"").to_string();
    let move_name = parts.get(4).map(|s| s.to_string());
    let attacker = parts.get(5).and_then(|s| Pokemon::parse(s));

    Ok(ServerMessage::Block {
        pokemon,
        effect,
        move_name,
        attacker,
    })
}

/// Parse |-notarget|POKEMON
pub fn parse_notarget(parts: &[&str]) -> Result<ServerMessage> {
    let pokemon = parts.get(2).and_then(|s| Pokemon::parse(s));
    Ok(ServerMessage::NoTarget(pokemon))
}

/// Parse |-miss|SOURCE|TARGET
pub fn parse_miss(parts: &[&str]) -> Result<ServerMessage> {
    let source = parse_pokemon(parts, 2)?;
    let target = parts.get(3).and_then(|s| Pokemon::parse(s));

    Ok(ServerMessage::Miss { source, target })
}

/// Parse |-damage|POKEMON|HP STATUS
pub fn parse_damage(parts: &[&str]) -> Result<ServerMessage> {
    let pokemon = parse_pokemon(parts, 2)?;
    let hp_status = parse_hp_status(parts, 3);

    Ok(ServerMessage::Damage { pokemon, hp_status })
}

/// Parse |-heal|POKEMON|HP STATUS
pub fn parse_heal(parts: &[&str]) -> Result<ServerMessage> {
    let pokemon = parse_pokemon(parts, 2)?;
    let hp_status = parse_hp_status(parts, 3);

    Ok(ServerMessage::Heal { pokemon, hp_status })
}

/// Parse |-sethp|POKEMON|HP
pub fn parse_sethp(parts: &[&str]) -> Result<ServerMessage> {
    let pokemon = parse_pokemon(parts, 2)?;
    let hp_status = parse_hp_status(parts, 3);

    Ok(ServerMessage::SetHp { pokemon, hp_status })
}

/// Parse |-status|POKEMON|STATUS
pub fn parse_status(parts: &[&str]) -> Result<ServerMessage> {
    let pokemon = parse_pokemon(parts, 2)?;
    let status = parts.get(3).unwrap_or(&"").to_string();

    Ok(ServerMessage::Status { pokemon, status })
}

/// Parse |-curestatus|POKEMON|STATUS
pub fn parse_curestatus(parts: &[&str]) -> Result<ServerMessage> {
    let pokemon = parse_pokemon(parts, 2)?;
    let status = parts.get(3).unwrap_or(&"").to_string();

    Ok(ServerMessage::CureStatus { pokemon, status })
}

/// Parse |-cureteam|POKEMON
pub fn parse_cureteam(parts: &[&str]) -> Result<ServerMessage> {
    let pokemon = parse_pokemon(parts, 2)?;
    Ok(ServerMessage::CureTeam(pokemon))
}

/// Parse |-boost|POKEMON|STAT|AMOUNT
pub fn parse_boost(parts: &[&str]) -> Result<ServerMessage> {
    let pokemon = parse_pokemon(parts, 2)?;
    let stat = parts
        .get(3)
        .and_then(|s| Stat::parse(s))
        .ok_or_else(|| anyhow::anyhow!("Missing stat"))?;
    let amount = parts
        .get(4)
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| anyhow::anyhow!("Missing amount"))?;

    Ok(ServerMessage::Boost {
        pokemon,
        stat,
        amount,
    })
}

/// Parse |-unboost|POKEMON|STAT|AMOUNT
pub fn parse_unboost(parts: &[&str]) -> Result<ServerMessage> {
    let pokemon = parse_pokemon(parts, 2)?;
    let stat = parts
        .get(3)
        .and_then(|s| Stat::parse(s))
        .ok_or_else(|| anyhow::anyhow!("Missing stat"))?;
    let amount = parts
        .get(4)
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| anyhow::anyhow!("Missing amount"))?;

    Ok(ServerMessage::Unboost {
        pokemon,
        stat,
        amount,
    })
}

/// Parse |-setboost|POKEMON|STAT|AMOUNT
pub fn parse_setboost(parts: &[&str]) -> Result<ServerMessage> {
    let pokemon = parse_pokemon(parts, 2)?;
    let stat = parts
        .get(3)
        .and_then(|s| Stat::parse(s))
        .ok_or_else(|| anyhow::anyhow!("Missing stat"))?;
    let amount = parts
        .get(4)
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| anyhow::anyhow!("Missing amount"))?;

    Ok(ServerMessage::SetBoost {
        pokemon,
        stat,
        amount,
    })
}

/// Parse |-swapboost|SOURCE|TARGET|STATS
pub fn parse_swapboost(parts: &[&str]) -> Result<ServerMessage> {
    let source = parse_pokemon(parts, 2)?;
    let target = parse_pokemon(parts, 3)?;
    let stats: Vec<Stat> = parts
        .get(4)
        .map(|s| s.split(',').filter_map(|s| Stat::parse(s.trim())).collect())
        .unwrap_or_default();

    Ok(ServerMessage::SwapBoost {
        source,
        target,
        stats,
    })
}

/// Parse |-invertboost|POKEMON
pub fn parse_invertboost(parts: &[&str]) -> Result<ServerMessage> {
    let pokemon = parse_pokemon(parts, 2)?;
    Ok(ServerMessage::InvertBoost(pokemon))
}

/// Parse |-clearboost|POKEMON
pub fn parse_clearboost(parts: &[&str]) -> Result<ServerMessage> {
    let pokemon = parse_pokemon(parts, 2)?;
    Ok(ServerMessage::ClearBoost(pokemon))
}

/// Parse |-clearallboost
pub fn parse_clearallboost(_parts: &[&str]) -> Result<ServerMessage> {
    Ok(ServerMessage::ClearAllBoost)
}

/// Parse |-clearpositiveboost|TARGET|POKEMON|EFFECT
pub fn parse_clearpositiveboost(parts: &[&str]) -> Result<ServerMessage> {
    let target = parse_pokemon(parts, 2)?;
    let source = parse_pokemon(parts, 3)?;
    let effect = parts.get(4).unwrap_or(&"").to_string();

    Ok(ServerMessage::ClearPositiveBoost {
        target,
        source,
        effect,
    })
}

/// Parse |-clearnegativeboost|POKEMON
pub fn parse_clearnegativeboost(parts: &[&str]) -> Result<ServerMessage> {
    let pokemon = parse_pokemon(parts, 2)?;
    Ok(ServerMessage::ClearNegativeBoost(pokemon))
}

/// Parse |-copyboost|SOURCE|TARGET
pub fn parse_copyboost(parts: &[&str]) -> Result<ServerMessage> {
    let source = parse_pokemon(parts, 2)?;
    let target = parse_pokemon(parts, 3)?;

    Ok(ServerMessage::CopyBoost { source, target })
}

/// Parse |-weather|WEATHER
pub fn parse_weather(parts: &[&str]) -> Result<ServerMessage> {
    let weather = parts.get(2).unwrap_or(&"none").to_string();
    let upkeep = parts.iter().any(|p| *p == "[upkeep]");

    Ok(ServerMessage::Weather { weather, upkeep })
}

/// Parse |-fieldstart|CONDITION
pub fn parse_fieldstart(parts: &[&str]) -> Result<ServerMessage> {
    let condition = parts.get(2).unwrap_or(&"").to_string();
    Ok(ServerMessage::FieldStart(condition))
}

/// Parse |-fieldend|CONDITION
pub fn parse_fieldend(parts: &[&str]) -> Result<ServerMessage> {
    let condition = parts.get(2).unwrap_or(&"").to_string();
    Ok(ServerMessage::FieldEnd(condition))
}

/// Parse |-sidestart|SIDE|CONDITION
pub fn parse_sidestart(parts: &[&str]) -> Result<ServerMessage> {
    let side = parts
        .get(2)
        .and_then(|s| Side::parse(s))
        .ok_or_else(|| anyhow::anyhow!("Missing side"))?;
    let condition = parts.get(3).unwrap_or(&"").to_string();

    Ok(ServerMessage::SideStart { side, condition })
}

/// Parse |-sideend|SIDE|CONDITION
pub fn parse_sideend(parts: &[&str]) -> Result<ServerMessage> {
    let side = parts
        .get(2)
        .and_then(|s| Side::parse(s))
        .ok_or_else(|| anyhow::anyhow!("Missing side"))?;
    let condition = parts.get(3).unwrap_or(&"").to_string();

    Ok(ServerMessage::SideEnd { side, condition })
}

/// Parse |-swapsideconditions
pub fn parse_swapsideconditions(_parts: &[&str]) -> Result<ServerMessage> {
    Ok(ServerMessage::SwapSideConditions)
}

/// Parse |-start|POKEMON|EFFECT
pub fn parse_start(parts: &[&str]) -> Result<ServerMessage> {
    let pokemon = parse_pokemon(parts, 2)?;
    let effect = parts.get(3).unwrap_or(&"").to_string();

    Ok(ServerMessage::VolatileStart { pokemon, effect })
}

/// Parse |-end|POKEMON|EFFECT
pub fn parse_end(parts: &[&str]) -> Result<ServerMessage> {
    let pokemon = parse_pokemon(parts, 2)?;
    let effect = parts.get(3).unwrap_or(&"").to_string();

    Ok(ServerMessage::VolatileEnd { pokemon, effect })
}

/// Parse |-crit|POKEMON
pub fn parse_crit(parts: &[&str]) -> Result<ServerMessage> {
    let pokemon = parse_pokemon(parts, 2)?;
    Ok(ServerMessage::Crit(pokemon))
}

/// Parse |-supereffective|POKEMON
pub fn parse_supereffective(parts: &[&str]) -> Result<ServerMessage> {
    let pokemon = parse_pokemon(parts, 2)?;
    Ok(ServerMessage::SuperEffective(pokemon))
}

/// Parse |-resisted|POKEMON
pub fn parse_resisted(parts: &[&str]) -> Result<ServerMessage> {
    let pokemon = parse_pokemon(parts, 2)?;
    Ok(ServerMessage::Resisted(pokemon))
}

/// Parse |-immune|POKEMON
pub fn parse_immune(parts: &[&str]) -> Result<ServerMessage> {
    let pokemon = parse_pokemon(parts, 2)?;
    Ok(ServerMessage::Immune(pokemon))
}

/// Parse |-item|POKEMON|ITEM with optional [from]EFFECT
pub fn parse_item(parts: &[&str]) -> Result<ServerMessage> {
    let pokemon = parse_pokemon(parts, 2)?;
    let item = parts.get(3).unwrap_or(&"").to_string();
    let from = parts
        .iter()
        .find_map(|p| p.strip_prefix("[from] ").map(|s| s.to_string()));

    Ok(ServerMessage::Item { pokemon, item, from })
}

/// Parse |-enditem|POKEMON|ITEM with optional [from]EFFECT or [eat]
pub fn parse_enditem(parts: &[&str]) -> Result<ServerMessage> {
    let pokemon = parse_pokemon(parts, 2)?;
    let item = parts.get(3).unwrap_or(&"").to_string();
    let from = parts
        .iter()
        .find_map(|p| p.strip_prefix("[from] ").map(|s| s.to_string()));
    let eat = parts.iter().any(|p| *p == "[eat]");

    Ok(ServerMessage::EndItem {
        pokemon,
        item,
        from,
        eat,
    })
}

/// Parse |-ability|POKEMON|ABILITY with optional [from]EFFECT
pub fn parse_ability(parts: &[&str]) -> Result<ServerMessage> {
    let pokemon = parse_pokemon(parts, 2)?;
    let ability = parts.get(3).unwrap_or(&"").to_string();
    let from = parts
        .iter()
        .find_map(|p| p.strip_prefix("[from] ").map(|s| s.to_string()));

    Ok(ServerMessage::Ability {
        pokemon,
        ability,
        from,
    })
}

/// Parse |-endability|POKEMON
pub fn parse_endability(parts: &[&str]) -> Result<ServerMessage> {
    let pokemon = parse_pokemon(parts, 2)?;
    Ok(ServerMessage::EndAbility(pokemon))
}

/// Parse |-transform|POKEMON|SPECIES
pub fn parse_transform(parts: &[&str]) -> Result<ServerMessage> {
    let pokemon = parse_pokemon(parts, 2)?;
    let species = parts.get(3).unwrap_or(&"").to_string();

    Ok(ServerMessage::Transform { pokemon, species })
}

/// Parse |-mega|POKEMON|MEGASTONE
pub fn parse_mega(parts: &[&str]) -> Result<ServerMessage> {
    let pokemon = parse_pokemon(parts, 2)?;
    let megastone = parts.get(3).unwrap_or(&"").to_string();

    Ok(ServerMessage::Mega { pokemon, megastone })
}

/// Parse |-primal|POKEMON
pub fn parse_primal(parts: &[&str]) -> Result<ServerMessage> {
    let pokemon = parse_pokemon(parts, 2)?;
    Ok(ServerMessage::Primal(pokemon))
}

/// Parse |-burst|POKEMON|SPECIES|ITEM
pub fn parse_burst(parts: &[&str]) -> Result<ServerMessage> {
    let pokemon = parse_pokemon(parts, 2)?;
    let species = parts.get(3).unwrap_or(&"").to_string();
    let item = parts.get(4).unwrap_or(&"").to_string();

    Ok(ServerMessage::Burst {
        pokemon,
        species,
        item,
    })
}

/// Parse |-zpower|POKEMON
pub fn parse_zpower(parts: &[&str]) -> Result<ServerMessage> {
    let pokemon = parse_pokemon(parts, 2)?;
    Ok(ServerMessage::ZPower(pokemon))
}

/// Parse |-zbroken|POKEMON
pub fn parse_zbroken(parts: &[&str]) -> Result<ServerMessage> {
    let pokemon = parse_pokemon(parts, 2)?;
    Ok(ServerMessage::ZBroken(pokemon))
}

/// Parse |-activate|EFFECT (with optional Pokemon and other fields)
pub fn parse_activate(parts: &[&str]) -> Result<ServerMessage> {
    // First part might be a Pokemon or an effect
    let pokemon = parts.get(2).and_then(|s| Pokemon::parse(s));
    let effect = if pokemon.is_some() {
        parts.get(3).unwrap_or(&"").to_string()
    } else {
        parts.get(2).unwrap_or(&"").to_string()
    };

    Ok(ServerMessage::Activate { pokemon, effect })
}

/// Parse |-hint|MESSAGE
pub fn parse_hint(parts: &[&str]) -> Result<ServerMessage> {
    let message = parts.get(2).unwrap_or(&"").to_string();
    Ok(ServerMessage::Hint(message))
}

/// Parse |-center
pub fn parse_center(_parts: &[&str]) -> Result<ServerMessage> {
    Ok(ServerMessage::Center)
}

/// Parse |-message|MESSAGE
pub fn parse_message(parts: &[&str]) -> Result<ServerMessage> {
    let message = parts.get(2).unwrap_or(&"").to_string();
    Ok(ServerMessage::Message(message))
}

/// Parse |-combine
pub fn parse_combine(_parts: &[&str]) -> Result<ServerMessage> {
    Ok(ServerMessage::Combine)
}

/// Parse |-waiting|SOURCE|TARGET
pub fn parse_waiting(parts: &[&str]) -> Result<ServerMessage> {
    let source = parse_pokemon(parts, 2)?;
    let target = parse_pokemon(parts, 3)?;

    Ok(ServerMessage::Waiting { source, target })
}

/// Parse |-prepare|ATTACKER|MOVE or |-prepare|ATTACKER|MOVE|DEFENDER
pub fn parse_prepare(parts: &[&str]) -> Result<ServerMessage> {
    let attacker = parse_pokemon(parts, 2)?;
    let move_name = parts.get(3).unwrap_or(&"").to_string();
    let defender = parts.get(4).and_then(|s| Pokemon::parse(s));

    Ok(ServerMessage::Prepare {
        attacker,
        move_name,
        defender,
    })
}

/// Parse |-mustrecharge|POKEMON
pub fn parse_mustrecharge(parts: &[&str]) -> Result<ServerMessage> {
    let pokemon = parse_pokemon(parts, 2)?;
    Ok(ServerMessage::MustRecharge(pokemon))
}

/// Parse |-nothing
pub fn parse_nothing(_parts: &[&str]) -> Result<ServerMessage> {
    Ok(ServerMessage::Nothing)
}

/// Parse |-hitcount|POKEMON|NUM
pub fn parse_hitcount(parts: &[&str]) -> Result<ServerMessage> {
    let pokemon = parse_pokemon(parts, 2)?;
    let count = parts
        .get(3)
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| anyhow::anyhow!("Missing hit count"))?;

    Ok(ServerMessage::HitCount { pokemon, count })
}

/// Parse |-singlemove|POKEMON|MOVE
pub fn parse_singlemove(parts: &[&str]) -> Result<ServerMessage> {
    let pokemon = parse_pokemon(parts, 2)?;
    let move_name = parts.get(3).unwrap_or(&"").to_string();

    Ok(ServerMessage::SingleMove { pokemon, move_name })
}

/// Parse |-singleturn|POKEMON|MOVE
pub fn parse_singleturn(parts: &[&str]) -> Result<ServerMessage> {
    let pokemon = parse_pokemon(parts, 2)?;
    let move_name = parts.get(3).unwrap_or(&"").to_string();

    Ok(ServerMessage::SingleTurn { pokemon, move_name })
}
