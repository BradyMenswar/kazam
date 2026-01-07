use super::{ChallengeState, Format, FormatSection, SearchState, ServerMessage, User};
use crate::ParseError;
use anyhow::Result;

pub fn parse_challstr(parts: &[&str]) -> Result<ServerMessage> {
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

pub fn parse_updateuser(parts: &[&str]) -> Result<ServerMessage> {
    if parts.len() < 4 {
        return Err(ParseError::MissingField("updateuser fields".to_string()).into());
    }

    let user = User::parse(parts[2])
        .ok_or_else(|| ParseError::InvalidFormat("invalid user format".to_string()))?;

    let named = parts[3] == "1";
    let avatar = parts.get(4).unwrap_or(&"").to_string();

    Ok(ServerMessage::UpdateUser { user, named, avatar })
}

pub fn parse_nametaken(parts: &[&str]) -> Result<ServerMessage> {
    if parts.len() < 4 {
        return Err(ParseError::MissingField("nametaken fields".to_string()).into());
    }

    Ok(ServerMessage::NameTaken {
        username: parts[2].to_string(),
        message: parts[3..].join("|"),
    })
}

pub fn parse_popup(parts: &[&str]) -> Result<ServerMessage> {
    if parts.len() < 3 {
        return Err(ParseError::MissingField("popup message".to_string()).into());
    }

    // MESSAGE can contain | characters
    Ok(ServerMessage::Popup(parts[2..].join("|")))
}

pub fn parse_pm(parts: &[&str]) -> Result<ServerMessage> {
    if parts.len() < 5 {
        return Err(ParseError::MissingField("pm fields".to_string()).into());
    }

    let sender = User::parse(parts[2])
        .ok_or_else(|| ParseError::InvalidFormat("invalid sender format".to_string()))?;
    let receiver = User::parse(parts[3])
        .ok_or_else(|| ParseError::InvalidFormat("invalid receiver format".to_string()))?;

    // MESSAGE can contain | characters
    let message = parts[4..].join("|");

    Ok(ServerMessage::Pm {
        sender,
        receiver,
        message,
    })
}

pub fn parse_usercount(parts: &[&str]) -> Result<ServerMessage> {
    if parts.len() < 3 {
        return Err(ParseError::MissingField("usercount value".to_string()).into());
    }

    let count = parts[2]
        .parse::<u32>()
        .map_err(|_| ParseError::InvalidFormat("invalid usercount".to_string()))?;

    Ok(ServerMessage::Usercount(count))
}

pub fn parse_formats(parts: &[&str]) -> Result<ServerMessage> {
    let mut sections = Vec::new();
    let mut current_section: Option<FormatSection> = None;

    // parts[0] is empty, parts[1] is "formats", parts[2..] is the format list
    // Sections start with ",#" where # is the column number
    // Empty parts indicate section boundaries (from ||)
    for part in parts.iter().skip(2) {
        if part.is_empty() {
            // Section boundary - save current section if any
            if let Some(section) = current_section.take() {
                sections.push(section);
            }
            continue;
        }

        // Check if this is a section header (starts with ,)
        if let Some(col_str) = part.strip_prefix(',') {
            // Save previous section
            if let Some(section) = current_section.take() {
                sections.push(section);
            }

            // Parse column number
            if let Ok(column) = col_str.parse::<u32>() {
                current_section = Some(FormatSection {
                    column,
                    name: String::new(),
                    formats: Vec::new(),
                });
            }
            continue;
        }

        // If we have a current section with empty name, this is the section name
        if let Some(ref mut section) = current_section {
            if section.name.is_empty() {
                section.name = part.to_string();
                continue;
            }

            // Otherwise it's a format entry
            section.formats.push(parse_format_entry(part));
        }
    }

    // Don't forget the last section
    if let Some(section) = current_section {
        sections.push(section);
    }

    Ok(ServerMessage::Formats(sections))
}

fn parse_format_entry(entry: &str) -> Format {
    // Format entries end with ,HEX where HEX is display flags
    if let Some((name, hex)) = entry.rsplit_once(',') {
        let flags = u8::from_str_radix(hex, 16).unwrap_or(0);
        Format {
            name: name.to_string(),
            random_team: flags & 1 != 0,
            search_show: flags & 2 != 0,
            challenge_show: flags & 4 != 0,
            tournament_show: flags & 8 != 0,
            level_50: flags & 16 != 0,
            best_of: flags & 64 != 0,
            tera_preview: flags & 128 != 0,
        }
    } else {
        Format {
            name: entry.to_string(),
            random_team: false,
            search_show: false,
            challenge_show: false,
            tournament_show: false,
            level_50: false,
            best_of: false,
            tera_preview: false,
        }
    }
}

pub fn parse_updatesearch(parts: &[&str]) -> Result<ServerMessage> {
    if parts.len() < 3 {
        return Err(ParseError::MissingField("updatesearch json".to_string()).into());
    }

    // JSON can contain | characters
    let json_str = parts[2..].join("|");
    let state: SearchState = serde_json::from_str(&json_str)
        .map_err(|e| ParseError::InvalidFormat(format!("invalid updatesearch json: {}", e)))?;

    Ok(ServerMessage::UpdateSearch(state))
}

pub fn parse_updatechallenges(parts: &[&str]) -> Result<ServerMessage> {
    if parts.len() < 3 {
        return Err(ParseError::MissingField("updatechallenges json".to_string()).into());
    }

    // JSON can contain | characters
    let json_str = parts[2..].join("|");
    let state: ChallengeState = serde_json::from_str(&json_str)
        .map_err(|e| ParseError::InvalidFormat(format!("invalid updatechallenges json: {}", e)))?;

    Ok(ServerMessage::UpdateChallenges(state))
}
