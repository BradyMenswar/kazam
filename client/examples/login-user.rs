use anyhow::Result;
use kazam_client::Client;

#[tokio::main]
async fn main() -> Result<()> {
    let username = std::env::var("PS_USERNAME").expect("Set PS_USERNAME environment variable");
    let password = std::env::var("PS_PASSWORD").expect("Set PS_PASSWORD environment variable");

    let mut client = Client::connect_default().await?;
    println!("Connected.");

    let confirmed = client.login_with_password(&username, &password).await?;
    println!("Logged in as: {}", confirmed);

    Ok(())
}
