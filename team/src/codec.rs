use crate::error::TeamError;
use crate::model::{
    PokemonSet, StatLine, Team, default_dynamax_level, default_happiness, default_level,
};

pub struct Teams;

impl Teams {
    pub fn unpack(packed_team: &str) -> Result<Team, TeamError> {
        if packed_team.trim().is_empty() {
            return Ok(Vec::new());
        }

        packed_team
            .split(']')
            .filter(|chunk| !chunk.trim().is_empty())
            .map(parse_packed_set)
            .collect()
    }

    pub fn pack(team: &[PokemonSet]) -> String {
        team.iter().map(pack_set).collect::<Vec<_>>().join("]")
    }

    pub fn import(input: &str) -> Result<Team, TeamError> {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return Ok(Vec::new());
        }

        if trimmed.starts_with('[') {
            return Ok(serde_json::from_str(trimmed)?);
        }

        if trimmed.starts_with('{') {
            let set: PokemonSet = serde_json::from_str(trimmed)?;
            return Ok(vec![set]);
        }

        if looks_like_export(trimmed) {
            return parse_export(trimmed);
        }

        Self::unpack(trimmed)
    }

    pub fn export(team: &[PokemonSet]) -> String {
        team.iter()
            .map(Self::export_set)
            .collect::<Vec<_>>()
            .join("\n\n")
    }

    pub fn export_set(set: &PokemonSet) -> String {
        let mut lines = vec![export_header(set)];

        if !set.ability.is_empty() {
            lines.push(format!("Ability: {}", set.ability));
        }

        if !set.evs.is_zero() {
            lines.push(format!("EVs: {}", format_spread(&set.evs)));
        }

        if !set.nature.is_empty() {
            lines.push(format!("{} Nature", set.nature));
        }

        if !set.ivs.is_all(31) {
            lines.push(format!("IVs: {}", format_spread(&set.ivs)));
        }

        if set.shiny {
            lines.push("Shiny: Yes".to_string());
        }

        if set.level != default_level() {
            lines.push(format!("Level: {}", set.level));
        }

        if set.happiness != default_happiness() {
            lines.push(format!("Happiness: {}", set.happiness));
        }

        if !set.pokeball.is_empty() {
            lines.push(format!("Pokeball: {}", set.pokeball));
        }

        if !set.hidden_power_type.is_empty() {
            lines.push(format!("Hidden Power Type: {}", set.hidden_power_type));
        }

        if set.gigantamax {
            lines.push("Gigantamax: Yes".to_string());
        }

        if set.dynamax_level != default_dynamax_level() {
            lines.push(format!("Dynamax Level: {}", set.dynamax_level));
        }

        if !set.tera_type.is_empty() {
            lines.push(format!("Tera Type: {}", set.tera_type));
        }

        for move_name in &set.moves {
            lines.push(format!("- {}", move_name));
        }

        lines.join("\n")
    }
}

fn pack_set(set: &PokemonSet) -> String {
    let nickname = if set.name.is_empty() {
        set.species.clone()
    } else {
        set.name.clone()
    };
    let species = if set.name.is_empty() || ids_equal(&set.name, &set.species) {
        String::new()
    } else {
        set.species.clone()
    };

    let extras = pack_extras(set);

    [
        nickname,
        species,
        to_id(&set.item),
        pack_ability(&set.ability),
        set.moves.iter().map(|m| to_id(m)).collect::<Vec<_>>().join(","),
        set.nature.clone(),
        pack_spread(&set.evs, 0),
        set.gender.clone(),
        pack_spread(&set.ivs, 31),
        if set.shiny { "S".to_string() } else { String::new() },
        if set.level == default_level() {
            String::new()
        } else {
            set.level.to_string()
        },
        extras,
    ]
    .join("|")
}

fn pack_ability(ability: &str) -> String {
    match ability {
        "0" | "1" | "H" => ability.to_string(),
        _ => to_id(ability),
    }
}

fn pack_spread(spread: &StatLine, default_value: u16) -> String {
    let values = [
        spread.hp,
        spread.atk,
        spread.def,
        spread.spa,
        spread.spd,
        spread.spe,
    ];

    if values.iter().all(|value| *value == default_value) {
        return String::new();
    }

    values
        .iter()
        .map(|value| {
            if *value == default_value {
                String::new()
            } else {
                value.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn pack_extras(set: &PokemonSet) -> String {
    let mut extras = vec![
        if set.happiness == default_happiness() {
            String::new()
        } else {
            set.happiness.to_string()
        },
        if set.pokeball.is_empty() || ids_equal(&set.pokeball, "Pokeball") {
            String::new()
        } else {
            to_id(&set.pokeball)
        },
        set.hidden_power_type.clone(),
        if set.gigantamax {
            "G".to_string()
        } else {
            String::new()
        },
        if set.dynamax_level == default_dynamax_level() {
            String::new()
        } else {
            set.dynamax_level.to_string()
        },
        set.tera_type.clone(),
    ];

    while extras.last().is_some_and(|value| value.is_empty()) {
        extras.pop();
    }

    extras.join(",")
}

fn parse_packed_set(chunk: &str) -> Result<PokemonSet, TeamError> {
    let mut fields = chunk.split('|').map(str::to_string).collect::<Vec<_>>();
    if fields.len() < 12 {
        fields.resize(12, String::new());
    }

    let name_or_species = fields[0].clone();
    let mut set = PokemonSet {
        name: String::new(),
        species: if fields[1].is_empty() {
            name_or_species.clone()
        } else {
            fields[1].clone()
        },
        item: fields[2].clone(),
        ability: fields[3].clone(),
        moves: parse_moves(&fields[4]),
        nature: fields[5].clone(),
        evs: parse_spread(&fields[6], 0)?,
        gender: fields[7].clone(),
        ivs: parse_spread(&fields[8], 31)?,
        shiny: fields[9] == "S",
        level: if fields[10].is_empty() {
            default_level()
        } else {
            fields[10].parse().map_err(|_| TeamError::InvalidPacked)?
        },
        ..PokemonSet::default()
    };

    if !fields[1].is_empty() && !ids_equal(&name_or_species, &set.species) {
        set.name = name_or_species;
    }

    let extras = fields[11].split(',').collect::<Vec<_>>();
    if let Some(value) = extras.first().filter(|value| !value.is_empty()) {
        set.happiness = value.parse().map_err(|_| TeamError::InvalidPacked)?;
    }
    if let Some(value) = extras.get(1).filter(|value| !value.is_empty()) {
        set.pokeball = (*value).to_string();
    }
    if let Some(value) = extras.get(2).filter(|value| !value.is_empty()) {
        set.hidden_power_type = (*value).to_string();
    }
    if let Some(value) = extras.get(3).filter(|value| !value.is_empty()) {
        set.gigantamax = *value == "G";
    }
    if let Some(value) = extras.get(4).filter(|value| !value.is_empty()) {
        set.dynamax_level = value.parse().map_err(|_| TeamError::InvalidPacked)?;
    }
    if let Some(value) = extras.get(5).filter(|value| !value.is_empty()) {
        set.tera_type = (*value).to_string();
    }

    Ok(set)
}

fn parse_export(input: &str) -> Result<Team, TeamError> {
    let mut team = Vec::new();
    let mut current = None::<PokemonSet>;

    for (line_index, raw_line) in input.lines().enumerate() {
        let line_number = line_index + 1;
        let line = raw_line.trim();

        if line.is_empty() {
            if let Some(set) = current.take()
                && (!set.species.is_empty() || !set.moves.is_empty())
            {
                team.push(set);
            }
            continue;
        }

        let set = current.get_or_insert_with(PokemonSet::default);

        if set.species.is_empty() && !is_property_line(line) && !line.starts_with("- ") {
            parse_header_line(line, set);
            continue;
        }

        if let Some(value) = line.strip_prefix("Ability: ") {
            set.ability = value.to_string();
            continue;
        }
        if let Some(value) = line.strip_prefix("EVs: ") {
            set.evs = parse_named_spread(value)?;
            continue;
        }
        if let Some(value) = line.strip_prefix("IVs: ") {
            set.ivs = parse_named_spread(value)?;
            continue;
        }
        if let Some(value) = line.strip_suffix(" Nature") {
            set.nature = value.to_string();
            continue;
        }
        if let Some(value) = line.strip_prefix("Level: ") {
            set.level = value.parse().map_err(|_| TeamError::InvalidExport {
                line_number,
                message: "invalid level".to_string(),
            })?;
            continue;
        }
        if let Some(value) = line.strip_prefix("Shiny: ") {
            set.shiny = value.eq_ignore_ascii_case("yes");
            continue;
        }
        if let Some(value) = line.strip_prefix("Happiness: ") {
            set.happiness = value.parse().map_err(|_| TeamError::InvalidExport {
                line_number,
                message: "invalid happiness".to_string(),
            })?;
            continue;
        }
        if let Some(value) = line.strip_prefix("Pokeball: ") {
            set.pokeball = value.to_string();
            continue;
        }
        if let Some(value) = line.strip_prefix("Hidden Power Type: ") {
            set.hidden_power_type = value.to_string();
            continue;
        }
        if let Some(value) = line.strip_prefix("Gigantamax: ") {
            set.gigantamax = value.eq_ignore_ascii_case("yes");
            continue;
        }
        if let Some(value) = line.strip_prefix("Dynamax Level: ") {
            set.dynamax_level = value.parse().map_err(|_| TeamError::InvalidExport {
                line_number,
                message: "invalid dynamax level".to_string(),
            })?;
            continue;
        }
        if let Some(value) = line.strip_prefix("Tera Type: ") {
            set.tera_type = value.to_string();
            continue;
        }
        if let Some(value) = line.strip_prefix("- ") {
            set.moves.push(value.to_string());
            continue;
        }

        return Err(TeamError::InvalidExport {
            line_number,
            message: format!("unrecognized line `{}`", line),
        });
    }

    if let Some(set) = current.take()
        && (!set.species.is_empty() || !set.moves.is_empty())
    {
        team.push(set);
    }

    Ok(team)
}

fn parse_header_line(line: &str, set: &mut PokemonSet) {
    let (head, item) = line
        .split_once(" @ ")
        .map_or((line, None), |(head, item)| (head, Some(item)));

    if let Some(item) = item {
        set.item = item.to_string();
    }

    let (head, gender) = if let Some(stripped) = head.strip_suffix(" (M)") {
        (stripped, "M")
    } else if let Some(stripped) = head.strip_suffix(" (F)") {
        (stripped, "F")
    } else {
        (head, "")
    };
    set.gender = gender.to_string();

    if let Some((name, species)) = parse_nickname_species(head) {
        set.name = if ids_equal(name, species) {
            String::new()
        } else {
            name.to_string()
        };
        set.species = species.to_string();
    } else {
        set.species = head.to_string();
    }
}

fn parse_nickname_species(head: &str) -> Option<(&str, &str)> {
    let (name, rest) = head.split_once(" (")?;
    let species = rest.strip_suffix(')')?;
    Some((name, species))
}

fn parse_named_spread(value: &str) -> Result<StatLine, TeamError> {
    let mut spread = StatLine::default();

    for chunk in value.split('/') {
        let chunk = chunk.trim();
        let (amount, stat_name) = chunk.split_once(' ').ok_or_else(|| {
            TeamError::InvalidExport {
                line_number: 0,
                message: format!("invalid spread segment `{}`", chunk),
            }
        })?;
        let amount = amount.parse::<u16>().map_err(|_| TeamError::InvalidExport {
            line_number: 0,
            message: format!("invalid spread amount `{}`", amount),
        })?;

        match stat_name {
            "HP" => spread.hp = amount,
            "Atk" => spread.atk = amount,
            "Def" => spread.def = amount,
            "SpA" => spread.spa = amount,
            "SpD" => spread.spd = amount,
            "Spe" => spread.spe = amount,
            other => return Err(TeamError::UnknownStat(other.to_string())),
        }
    }

    Ok(spread)
}

fn parse_spread(value: &str, default_value: u16) -> Result<StatLine, TeamError> {
    if value.is_empty() {
        return Ok(StatLine::all(default_value));
    }

    let mut values = [default_value; 6];
    for (index, part) in value.split(',').take(6).enumerate() {
        if !part.is_empty() {
            values[index] = part.parse().map_err(|_| TeamError::InvalidPacked)?;
        }
    }

    Ok(StatLine {
        hp: values[0],
        atk: values[1],
        def: values[2],
        spa: values[3],
        spd: values[4],
        spe: values[5],
    })
}

fn parse_moves(value: &str) -> Vec<String> {
    if value.is_empty() {
        return Vec::new();
    }

    value.split(',').map(|part| part.to_string()).collect()
}

fn format_spread(spread: &StatLine) -> String {
    let mut parts = Vec::new();
    if spread.hp != 0 {
        parts.push(format!("{} HP", spread.hp));
    }
    if spread.atk != 0 {
        parts.push(format!("{} Atk", spread.atk));
    }
    if spread.def != 0 {
        parts.push(format!("{} Def", spread.def));
    }
    if spread.spa != 0 {
        parts.push(format!("{} SpA", spread.spa));
    }
    if spread.spd != 0 {
        parts.push(format!("{} SpD", spread.spd));
    }
    if spread.spe != 0 {
        parts.push(format!("{} Spe", spread.spe));
    }
    parts.join(" / ")
}

fn export_header(set: &PokemonSet) -> String {
    let mut header = if set.name.is_empty() || ids_equal(&set.name, &set.species) {
        set.species.clone()
    } else {
        format!("{} ({})", set.name, set.species)
    };

    if !set.gender.is_empty() {
        header.push_str(&format!(" ({})", set.gender));
    }
    if !set.item.is_empty() {
        header.push_str(&format!(" @ {}", set.item));
    }

    header
}

fn looks_like_export(input: &str) -> bool {
    input.lines().any(|line| {
        let line = line.trim();
        line.starts_with("- ")
            || line.starts_with("Ability: ")
            || line.ends_with(" Nature")
            || line.starts_with("EVs: ")
            || line.starts_with("IVs: ")
    })
}

fn is_property_line(line: &str) -> bool {
    line.starts_with("Ability: ")
        || line.starts_with("EVs: ")
        || line.starts_with("IVs: ")
        || line.starts_with("Level: ")
        || line.starts_with("Shiny: ")
        || line.starts_with("Happiness: ")
        || line.starts_with("Pokeball: ")
        || line.starts_with("Hidden Power Type: ")
        || line.starts_with("Gigantamax: ")
        || line.starts_with("Dynamax Level: ")
        || line.starts_with("Tera Type: ")
        || line.ends_with(" Nature")
}

fn ids_equal(left: &str, right: &str) -> bool {
    to_id(left) == to_id(right)
}

fn to_id(value: &str) -> String {
    value
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .flat_map(|ch| ch.to_lowercase())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_set() -> PokemonSet {
        PokemonSet {
            species: "Articuno".to_string(),
            item: "Leftovers".to_string(),
            ability: "Pressure".to_string(),
            evs: StatLine {
                hp: 252,
                spa: 252,
                spd: 4,
                ..StatLine::default()
            },
            nature: "Modest".to_string(),
            ivs: StatLine {
                spa: 30,
                spd: 30,
                ..crate::default_ivs()
            },
            moves: vec![
                "Ice Beam".to_string(),
                "Hurricane".to_string(),
                "Substitute".to_string(),
                "Roost".to_string(),
            ],
            ..PokemonSet::default()
        }
    }

    #[test]
    fn test_pack_and_unpack_team() {
        let team = vec![sample_set()];
        let packed = Teams::pack(&team);
        let unpacked = Teams::unpack(&packed).unwrap();

        assert_eq!(packed, "Articuno||leftovers|pressure|icebeam,hurricane,substitute,roost|Modest|252,,,252,4,||,,,30,30,|||");
        assert_eq!(unpacked[0].species, "Articuno");
        assert_eq!(unpacked[0].item, "leftovers");
        assert_eq!(unpacked[0].moves, vec!["icebeam", "hurricane", "substitute", "roost"]);
        assert_eq!(unpacked[0].ivs.spa, 30);
        assert_eq!(unpacked[0].ivs.spd, 30);
    }

    #[test]
    fn test_import_export_format() {
        let exported = "Volbeat (M) @ Damp Rock\nAbility: Prankster\nEVs: 248 HP / 252 Def / 8 SpD\nBold Nature\n- Tail Glow\n- Baton Pass\n- Encore\n- Rain Dance";

        let team = Teams::import(exported).unwrap();

        assert_eq!(team.len(), 1);
        assert_eq!(team[0].species, "Volbeat");
        assert_eq!(team[0].gender, "M");
        assert_eq!(team[0].item, "Damp Rock");
        assert_eq!(team[0].evs.hp, 248);
        assert_eq!(team[0].moves[0], "Tail Glow");

        assert_eq!(Teams::export(&team), exported);
    }

    #[test]
    fn test_import_json() {
        let json = r#"[{"name":"","species":"Ludicolo","gender":"","item":"Life Orb","ability":"Swift Swim","evs":{"hp":4,"atk":0,"def":0,"spa":252,"spd":0,"spe":252},"nature":"Modest","moves":["Surf","Giga Drain","Ice Beam","Rain Dance"]}]"#;

        let team = Teams::import(json).unwrap();

        assert_eq!(team.len(), 1);
        assert_eq!(team[0].species, "Ludicolo");
        assert_eq!(team[0].item, "Life Orb");
        assert_eq!(team[0].evs.spa, 252);
        assert_eq!(team[0].ivs, crate::default_ivs());
    }

    #[test]
    fn test_pack_with_optional_extras() {
        let team = vec![PokemonSet {
            name: "PROBLEMS".to_string(),
            species: "Tyranitar".to_string(),
            ability: "Sand Stream".to_string(),
            moves: vec!["Rock Slide".to_string()],
            shiny: true,
            level: 76,
            happiness: 200,
            pokeball: "Ultra Ball".to_string(),
            hidden_power_type: "Ice".to_string(),
            gigantamax: true,
            dynamax_level: 7,
            tera_type: "Ghost".to_string(),
            ..PokemonSet::default()
        }];

        let packed = Teams::pack(&team);
        let unpacked = Teams::unpack(&packed).unwrap();

        assert_eq!(
            packed,
            "PROBLEMS|Tyranitar||sandstream|rockslide|||||S|76|200,ultraball,Ice,G,7,Ghost"
        );
        assert_eq!(unpacked[0].name, "PROBLEMS");
        assert_eq!(unpacked[0].species, "Tyranitar");
        assert_eq!(unpacked[0].hidden_power_type, "Ice");
        assert!(unpacked[0].gigantamax);
        assert_eq!(unpacked[0].dynamax_level, 7);
    }
}
