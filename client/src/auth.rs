use anyhow::{Result, anyhow};

const LOGIN_URL: &str = "https://play.pokemonshowdown.com/api/login";

/// Get an assertion token from the Pokemon Showdown login server
pub async fn get_assertion(username: &str, password: &str, challstr: &str) -> Result<String> {
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
