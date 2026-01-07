use anyhow::Result;
use kazam_client::{KazamClient, KazamHandle, KazamHandler, RoomState, SHOWDOWN_URL, User};

struct YoBot {
    handle: KazamHandle,
}

impl KazamHandler for YoBot {
    async fn on_challstr(&mut self, challstr: &str) {
        println!("Logging in...");
        self.handle
            .login("bmax117", "dragon117", challstr)
            .await
            .expect("Failed to login");
    }

    async fn on_logged_in(&mut self, user: &User) {
        println!("Logged in as: {}{}", user.rank, user.username);
        self.handle
            .join_room("overused")
            .expect("Failed to join room");
    }

    async fn on_name_taken(&mut self, username: &str, message: &str) {
        println!("Login failed for {}: {}", username, message);
    }

    async fn on_room_joined(&mut self, room: &RoomState) {
        println!(
            "Joined room: {} ({} users)",
            room.title.as_deref().unwrap_or(&room.id),
            room.users.len()
        );
    }

    async fn on_chat(
        &mut self,
        room_id: Option<&str>,
        user: &User,
        message: &str,
        _ts: Option<i64>,
    ) {
        // Log all messages
        if let Some(room) = room_id {
            println!("[{}] {}{}: {}", room, user.rank, user.username, message);

            // Respond to !yo
            if message == "!yo" {
                let response = format!("hi {}", user.username);
                self.handle.send_chat(room, &response).ok();
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("Connecting to Pokemon Showdown...");
    let mut client = KazamClient::connect(SHOWDOWN_URL).await?;
    println!("Connected.");

    let mut handler = YoBot {
        handle: client.handle(),
    };

    client.run(&mut handler).await
}
