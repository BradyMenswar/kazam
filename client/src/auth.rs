use anyhow::{Result, anyhow};
use kazam_protocol::{ClientCommand, ClientMessage};

use crate::KazamClient;

const LOGIN_URL: &str = "https://play.pokemonshowdown.com/api/login";

impl KazamClient {
    pub async fn login(&mut self, username: &str, password: &str, challstr: &str) -> Result<()> {
        let assertion = get_assertion(username, password, challstr).await?;

        let cmd = ClientMessage {
            room_id: Some(String::new()),
            command: ClientCommand::TrustedLogin {
                username: username.to_string(),
                assertion,
            },
        };

        self.send_raw(cmd.to_wire_format()).await
    }
}

async fn get_assertion(username: &str, password: &str, challstr: &str) -> Result<String> {
    let client = reqwest::Client::new();

    let params = [
        ("name", username),
        ("pass", password),
        ("challstr", challstr),
    ];

    let response = client.post(LOGIN_URL).query(&params).send().await?;
    let text = response.text().await?;

    // Response is prefixed with "]"
    let json_str = text.trim_start_matches(']');
    let json: serde_json::Value = serde_json::from_str(json_str)?;

    if let Some(assertion) = json.get("assertion").and_then(|v| v.as_str()) {
        if assertion.starts_with(";;") {
            // Error message
            return Err(anyhow!("Login failed: {}", &assertion[2..]));
        }
        Ok(assertion.to_string())
    } else {
        Err(anyhow!("Login response missing assertion"))
    }
}
