use super::{RoomType, ServerMessage, User};
use crate::ParseError;
use anyhow::Result;

pub fn parse_join(parts: &[&str], quiet: bool) -> Result<ServerMessage> {
    if parts.len() < 3 {
        return Err(ParseError::MissingField("join fields".to_string()).into());
    }

    let user = User::parse(parts[2])
        .ok_or_else(|| ParseError::InvalidFormat("invalid user format".to_string()))?;

    Ok(ServerMessage::Join { user, quiet })
}

pub fn parse_leave(parts: &[&str], quiet: bool) -> Result<ServerMessage> {
    if parts.len() < 3 {
        return Err(ParseError::MissingField("leave fields".to_string()).into());
    }

    let user = User::parse(parts[2])
        .ok_or_else(|| ParseError::InvalidFormat("invalid user format".to_string()))?;

    Ok(ServerMessage::Leave { user, quiet })
}

pub fn parse_init(parts: &[&str]) -> Result<ServerMessage> {
    if parts.len() < 3 {
        return Err(ParseError::MissingField("init fields".to_string()).into());
    }

    let room_type = match parts[2] {
        "chat" => RoomType::Chat,
        "battle" => RoomType::Battle,
        _ => return Err(ParseError::InvalidFormat(format!("unknown room type: {}", parts[2])).into()),
    };

    Ok(ServerMessage::Init(room_type))
}

pub fn parse_title(parts: &[&str]) -> Result<ServerMessage> {
    if parts.len() < 3 {
        return Err(ParseError::MissingField("title field".to_string()).into());
    }

    Ok(ServerMessage::Title(parts[2..].join("|")))
}

pub fn parse_users(parts: &[&str]) -> Result<ServerMessage> {
    if parts.len() < 3 {
        return Err(ParseError::MissingField("users field".to_string()).into());
    }

    // User list is comma-separated, first entry is the user count
    let user_list = parts[2];
    let users: Vec<User> = user_list
        .split(',')
        .skip(1) // First element is the count
        .filter_map(|u| User::parse(u.trim()))
        .collect();

    Ok(ServerMessage::Users(users))
}

pub fn parse_chat(parts: &[&str], timestamp: Option<i64>) -> Result<ServerMessage> {
    if parts.len() < 4 {
        return Err(ParseError::MissingField("chat fields".to_string()).into());
    }

    let user = User::parse(parts[2])
        .ok_or_else(|| ParseError::InvalidFormat("invalid user format".to_string()))?;

    // MESSAGE can contain | characters, so join everything after parts[2]
    let message = parts[3..].join("|");

    Ok(ServerMessage::Chat {
        user,
        message,
        timestamp,
    })
}

pub fn parse_timestamped_chat(parts: &[&str]) -> Result<ServerMessage> {
    // |c:|TIMESTAMP|USER|MESSAGE
    if parts.len() < 5 {
        return Err(ParseError::MissingField("timestamped chat fields".to_string()).into());
    }

    let timestamp = parts[2]
        .parse::<i64>()
        .map_err(|_| ParseError::InvalidFormat("invalid timestamp".to_string()))?;

    let user = User::parse(parts[3])
        .ok_or_else(|| ParseError::InvalidFormat("invalid user format".to_string()))?;

    // MESSAGE can contain | characters
    let message = parts[4..].join("|");

    Ok(ServerMessage::Chat {
        user,
        message,
        timestamp: Some(timestamp),
    })
}

pub fn parse_timestamp(parts: &[&str]) -> Result<ServerMessage> {
    if parts.len() < 3 {
        return Err(ParseError::MissingField("timestamp field".to_string()).into());
    }

    let timestamp = parts[2]
        .parse::<i64>()
        .map_err(|_| ParseError::InvalidFormat("invalid timestamp".to_string()))?;

    Ok(ServerMessage::Timestamp(timestamp))
}

pub fn parse_battle(parts: &[&str]) -> Result<ServerMessage> {
    if parts.len() < 5 {
        return Err(ParseError::MissingField("battle fields".to_string()).into());
    }

    let user1 = User::parse(parts[3])
        .ok_or_else(|| ParseError::InvalidFormat("invalid user1 format".to_string()))?;
    let user2 = User::parse(parts[4])
        .ok_or_else(|| ParseError::InvalidFormat("invalid user2 format".to_string()))?;

    Ok(ServerMessage::Battle {
        room_id: parts[2].to_string(),
        user1,
        user2,
    })
}

pub fn parse_notify(parts: &[&str]) -> Result<ServerMessage> {
    if parts.len() < 3 {
        return Err(ParseError::MissingField("notify title".to_string()).into());
    }

    let title = parts[2].to_string();
    let message = parts.get(3).filter(|s| !s.is_empty()).map(|s| s.to_string());
    let highlight_token = parts.get(4).filter(|s| !s.is_empty()).map(|s| s.to_string());

    Ok(ServerMessage::Notify {
        title,
        message,
        highlight_token,
    })
}

pub fn parse_name(parts: &[&str], quiet: bool) -> Result<ServerMessage> {
    if parts.len() < 4 {
        return Err(ParseError::MissingField("name fields".to_string()).into());
    }

    let user = User::parse(parts[2])
        .ok_or_else(|| ParseError::InvalidFormat("invalid user format".to_string()))?;

    Ok(ServerMessage::Name {
        user,
        old_id: parts[3].to_string(),
        quiet,
    })
}

pub fn parse_html(parts: &[&str]) -> Result<ServerMessage> {
    if parts.len() < 3 {
        return Err(ParseError::MissingField("html content".to_string()).into());
    }

    // HTML can contain | characters
    Ok(ServerMessage::Html(parts[2..].join("|")))
}

pub fn parse_uhtml(parts: &[&str]) -> Result<ServerMessage> {
    if parts.len() < 4 {
        return Err(ParseError::MissingField("uhtml fields".to_string()).into());
    }

    // HTML can contain | characters
    Ok(ServerMessage::Uhtml {
        name: parts[2].to_string(),
        html: parts[3..].join("|"),
    })
}

pub fn parse_uhtmlchange(parts: &[&str]) -> Result<ServerMessage> {
    if parts.len() < 4 {
        return Err(ParseError::MissingField("uhtmlchange fields".to_string()).into());
    }

    // HTML can contain | characters
    Ok(ServerMessage::UhtmlChange {
        name: parts[2].to_string(),
        html: parts[3..].join("|"),
    })
}
