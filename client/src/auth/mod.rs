use crate::Connection;
use anyhow::{Context, Result};
use kazam_protocol::{ClientCommand, ClientMessage, ServerMessage};

/// Client authentication state
#[derive(Debug, Clone)]
pub enum AuthState {
    /// Not yet authenticated
    Connected,
    /// Authenticated as a guest
    Guest { username: String },
    /// Authenticated with credentials
    Authenticated { username: String },
}

/// Login with username and password
pub async fn login_with_password(
    connection: &mut Connection,
    username: &str,
    password: &str,
) -> Result<String> {
    let challstr = wait_for_challstr(connection).await?;

    let assertion = authenticate(username, password, &challstr).await?;

    let login_message = ClientMessage {
        room_id: None,
        command: ClientCommand::TrustedLogin {
            username: username.to_string(),
            assertion,
        },
    };

    connection.send(login_message).await?;

    wait_for_login_confirmation(connection).await
}

/// Wait for the challstr message from the server
async fn wait_for_challstr(connection: &mut Connection) -> Result<String> {
    loop {
        if let Some(frame) = connection.next_frame().await? {
            for message in frame.messages {
                if let ServerMessage::Challstr(challstr) = message {
                    return Ok(challstr);
                }
            }
        } else {
            anyhow::bail!("Connection closed while waiting for challstr")
        }
    }
}

/// Wait for updateuser confirmation after login
async fn wait_for_login_confirmation(connection: &mut Connection) -> Result<String> {
    loop {
        if let Some(frame) = connection.next_frame().await? {
            for message in frame.messages {
                if let ServerMessage::UpdateUser { username, .. } = message {
                    return Ok(username);
                }
                if let ServerMessage::NameTaken { message, .. } = message {
                    anyhow::bail!("Login failed: {}", message);
                }
            }
        } else {
            anyhow::bail!("Connection closed while waiting for login confirmation");
        }
    }
}

/// Authenticate with the Pokemon Showdown login server
async fn authenticate(username: &str, password: &str, challstr: &str) -> Result<String> {
    let client = reqwest::Client::new();

    let response = client
        .post("https://play.pokemonshowdown.com/api/login")
        .form(&[
            ("name", username),
            ("pass", password),
            ("challstr", challstr),
        ])
        .send()
        .await
        .context("Failed to send login request")?;

    let body = response.text().await?;

    // Response starts with ']' followed by JSON
    let json_str = body
        .strip_prefix(']')
        .context("Invalid login response format")?;

    let json: serde_json::Value =
        serde_json::from_str(json_str).context("Failed to parse login response")?;

    if json.get("actionsuccess").and_then(|v| v.as_bool()) != Some(true) {
        let error = json
            .get("assertion")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown error");
        anyhow::bail!("Login failed: {}", error);
    }

    let assertion = json
        .get("assertion")
        .and_then(|v| v.as_str())
        .context("No assertion in login response")?
        .to_string();

    Ok(assertion)
}
