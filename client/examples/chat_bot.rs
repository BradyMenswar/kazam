use anyhow::Result;
use kazam_client::{KazamClient, KazamHandler, RoomId, SHOWDOWN_URL};

struct Chatbot {}

impl KazamHandler for Chatbot {
    async fn on_challstr(&mut self, client: &mut KazamClient, challstr: &str) {
        println!("Logging in...");
        client
            .login(
                "kazam-bot",
                "Backhand8-Princess1-Ranking0-Monstrous8",
                challstr,
            )
            .await
            .expect("Failed to login");
    }

    async fn on_update_user(
        &mut self,
        client: &mut KazamClient,
        username: &str,
        named: bool,
        _avatar: &str,
    ) {
        println!("Logged in as: {} (named: {})", username, named);
        if named {
            client
                .join_room("overused")
                .await
                .expect("Failed to join room");
        }
    }

    async fn on_name_taken(&mut self, _client: &mut KazamClient, username: &str, message: &str) {
        println!("Login failed for {}: {}", username, message);
    }

    async fn on_join(
        &mut self,
        _client: &mut KazamClient,
        username: &str,
        _quiet: bool,
        _away: bool,
    ) {
        println!("{} joined.", username);
    }

    // async fn on_raw(&mut self, _client: &mut KazamClient, room: Option<&str>, content: &str) {
    //     println!("[{}] {}", room.unwrap_or("global"), content);
    // }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("Connecting to PS server...");
    let mut client = KazamClient::init(SHOWDOWN_URL).await?;
    println!("Connected.");

    let mut handler = Chatbot {};

    client.run(&mut handler).await?;

    println!("Connection closed.");
    Ok(())
}
