use std::io::{self, Write};
use std::sync::{Arc, Mutex};

use anyhow::Result;
use kazam_client::{KazamClient, KazamHandle, KazamHandler, RoomState, SHOWDOWN_URL, User};
use tokio::io::{AsyncBufReadExt, BufReader};

struct CliChat {
    handle: KazamHandle,
    current_room: Arc<Mutex<Option<String>>>,
    credentials: Option<(String, String)>,
}

impl KazamHandler for CliChat {
    async fn on_challstr(&mut self, challstr: &str) {
        if let Some((username, password)) = &self.credentials {
            println!("Logging in as {}...", username);
            if let Err(e) = self.handle.login(username, password, challstr).await {
                println!("Login error: {}", e);
            }
        }
    }

    async fn on_logged_in(&mut self, user: &User) {
        println!("Logged in as: {}{}", user.rank, user.username);
        println!("Type /help for commands");
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
        // Auto-switch to newly joined room
        if let Ok(mut current) = self.current_room.lock() {
            *current = Some(room.id.clone());
        }
        println!("Switched to room: {}", room.id);
    }

    async fn on_join(&mut self, room_id: Option<&str>, user: &User, quiet: bool) {
        if !quiet {
            if let Some(room) = room_id {
                println!("[{}] {} joined", room, user.username);
            }
        }
    }

    async fn on_leave(&mut self, room_id: Option<&str>, user: &User, quiet: bool) {
        if !quiet {
            if let Some(room) = room_id {
                println!("[{}] {} left", room, user.username);
            }
        }
    }

    async fn on_chat(
        &mut self,
        room_id: Option<&str>,
        user: &User,
        message: &str,
        _ts: Option<i64>,
    ) {
        if let Some(room) = room_id {
            println!("[{}] {}{}: {}", room, user.rank, user.username, message);
        } else {
            println!("{}{}: {}", user.rank, user.username, message);
        }
    }
}

fn print_help() {
    println!("Commands:");
    println!("  /join <room>   - Join a room");
    println!("  /leave [room]  - Leave current or specified room");
    println!("  /room <room>   - Switch to a room");
    println!("  /rooms         - List joined rooms");
    println!("  /quit          - Exit");
    println!("  <message>      - Send message to current room");
}

async fn handle_input(
    line: &str,
    handle: &KazamHandle,
    current_room: &Arc<Mutex<Option<String>>>,
) -> bool {
    let line = line.trim();
    if line.is_empty() {
        return true;
    }

    if line.starts_with('/') {
        let parts: Vec<&str> = line[1..].splitn(2, ' ').collect();
        let cmd = parts[0];
        let arg = parts.get(1).map(|s| s.trim());

        match cmd {
            "help" => print_help(),
            "join" => {
                if let Some(room) = arg {
                    if let Err(e) = handle.join_room(room) {
                        println!("Error: {}", e);
                    }
                } else {
                    println!("Usage: /join <room>");
                }
            }
            "leave" => {
                let room = arg
                    .map(String::from)
                    .or_else(|| current_room.lock().ok()?.clone());
                if let Some(room) = room {
                    if let Err(e) = handle.leave_room(&room) {
                        println!("Error: {}", e);
                    } else {
                        println!("Left room: {}", room);
                        if let Ok(mut current) = current_room.lock() {
                            if current.as_ref() == Some(&room) {
                                *current = None;
                            }
                        }
                    }
                } else {
                    println!("Not in a room. Usage: /leave [room]");
                }
            }
            "room" => {
                if let Some(room) = arg {
                    if handle.in_room(room) {
                        if let Ok(mut current) = current_room.lock() {
                            *current = Some(room.to_string());
                        }
                        println!("Switched to room: {}", room);
                    } else {
                        println!("Not in room: {}", room);
                    }
                } else {
                    println!("Usage: /room <room>");
                }
            }
            "rooms" => {
                let rooms = handle.rooms();
                if rooms.is_empty() {
                    println!("Not in any rooms");
                } else {
                    let current = current_room.lock().ok().and_then(|c| c.clone());
                    println!("Joined rooms:");
                    for room in rooms {
                        let marker = if Some(&room) == current.as_ref() {
                            " *"
                        } else {
                            ""
                        };
                        println!("  {}{}", room, marker);
                    }
                }
            }
            "quit" | "exit" => return false,
            _ => println!("Unknown command: /{}. Type /help for commands.", cmd),
        }
    } else {
        // Send as chat message
        let room = current_room.lock().ok().and_then(|c| c.clone());
        if let Some(room) = room {
            if let Err(e) = handle.send_chat(&room, line) {
                println!("Error: {}", e);
            }
        } else {
            println!("No room selected. Use /join <room> first.");
        }
    }

    true
}

fn prompt_credentials() -> Result<(String, String)> {
    print!("Username: ");
    io::stdout().flush()?;
    let mut username = String::new();
    io::stdin().read_line(&mut username)?;

    print!("Password: ");
    io::stdout().flush()?;
    let mut password = String::new();
    io::stdin().read_line(&mut password)?;

    Ok((username.trim().to_string(), password.trim().to_string()))
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("Pokemon Showdown CLI Chat");
    println!("=========================");

    let credentials = prompt_credentials()?;
    if credentials.0.is_empty() {
        println!("Username required");
        return Ok(());
    }

    println!("\nConnecting to Pokemon Showdown...");
    let mut client = KazamClient::connect(SHOWDOWN_URL).await?;
    println!("Connected.\n");

    let handle = client.handle();
    let current_room: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));

    let mut handler = CliChat {
        handle: handle.clone(),
        current_room: current_room.clone(),
        credentials: Some(credentials),
    };

    // Spawn input handler
    let input_handle = handle.clone();
    let input_room = current_room.clone();
    tokio::spawn(async move {
        let stdin = BufReader::new(tokio::io::stdin());
        let mut lines = stdin.lines();

        while let Ok(Some(line)) = lines.next_line().await {
            if !handle_input(&line, &input_handle, &input_room).await {
                break;
            }
        }

        // Exit when input ends
        std::process::exit(0);
    });

    // Run the client
    client.run(&mut handler).await
}
