use anyhow::Result;
use kazam_client::{Client, ServerMessage};

const SHOWDOWN_URL: &str = "wss://sim3.psim.us/showdown/websocket";

#[tokio::main]
async fn main() -> Result<()> {
    let username = std::env::var("PS_USERNAME").ok();
    let password = std::env::var("PS_PASSWORD").ok();
    let room = std::env::var("PS_ROOM").unwrap_or_else(|_| "lobby".to_string());

    println!("Connecting to Pokemon Showdown...");
    let mut client = Client::connect(SHOWDOWN_URL).await?;
    println!("Connected!");

    while let Some(frame) = client.next_frame().await {
        let frame = frame?;

        let room_id = frame.room_id.as_deref().unwrap_or("global");

        for msg in frame.messages {
            match msg {
                ServerMessage::Challstr(_) => {
                    if let (Some(user), Some(pass)) = (&username, &password) {
                        println!("Logging in as {}...", user);
                        client.login(user, pass).await?;
                    } else {
                        println!("No credentials provided, running as guest");
                    }
                }

                ServerMessage::UpdateUser {
                    username,
                    named,
                    avatar: _,
                } => {
                    println!("Logged in as: {} (named: {})", username, named);
                    if named {
                        println!("Joining room: {}", room);
                        client.join_room(&room).await?;
                    }
                }

                ServerMessage::NameTaken { username, message } => {
                    println!("Login failed for {}: {}", username, message);
                }

                ServerMessage::Raw(content) => {
                    // Log raw messages for debugging
                    if std::env::var("PS_DEBUG").is_ok() {
                        println!("[{}] {}", room_id, content);
                    }
                }
            }
        }
    }

    println!("Connection closed");
    Ok(())
}
