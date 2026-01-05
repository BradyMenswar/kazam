//! Simple chat bot example using the Handler trait
//!
//! Usage:
//!   PS_USERNAME=your_username PS_PASSWORD=your_password cargo run --example chat_bot
//!
//! For guest mode (no login):
//!   cargo run --example chat_bot

use anyhow::Result;
use kazam_client::{async_trait, Client, Handler, Sender};

const SHOWDOWN_URL: &str = "wss://sim3.psim.us/showdown/websocket";

struct ChatBot {
    sender: Sender,
    username: Option<String>,
    password: Option<String>,
    room: String,
    debug: bool,
}

impl ChatBot {
    fn new(sender: Sender) -> Self {
        Self {
            sender,
            username: std::env::var("PS_USERNAME").ok(),
            password: std::env::var("PS_PASSWORD").ok(),
            room: std::env::var("PS_ROOM").unwrap_or_else(|_| "lobby".to_string()),
            debug: std::env::var("PS_DEBUG").is_ok(),
        }
    }
}

#[async_trait]
impl Handler for ChatBot {
    async fn on_challstr(&mut self, challstr: &str) {
        if let (Some(user), Some(pass)) = (&self.username, &self.password) {
            println!("Logging in as {}...", user);
            if let Err(e) = self.sender.login(user, pass, challstr).await {
                eprintln!("Login error: {}", e);
            }
        } else {
            println!("No credentials provided, running as guest");
        }
    }

    async fn on_update_user(&mut self, username: &str, logged_in: bool, _avatar: &str) {
        println!("Logged in as: {} (named: {})", username, logged_in);
        if logged_in {
            println!("Joining room: {}", self.room);
            if let Err(e) = self.sender.join_room(&self.room).await {
                eprintln!("Join room error: {}", e);
            }
        }
    }

    async fn on_name_taken(&mut self, username: &str, message: &str) {
        println!("Login failed for {}: {}", username, message);
    }

    async fn on_raw(&mut self, room: Option<&str>, content: &str) {
        if self.debug {
            println!("[{}] {}", room.unwrap_or("global"), content);
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("Connecting to Pokemon Showdown...");

    let client = Client::connect(SHOWDOWN_URL).await?;
    let (mut receiver, sender) = client.split();

    println!("Connected!");

    let mut handler = ChatBot::new(sender);
    receiver.run(&mut handler).await?;

    println!("Connection closed");
    Ok(())
}
