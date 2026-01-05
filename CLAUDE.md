# Pokemon Showdown Client Toolkit

## Overview

A Rust crate providing the foundation for building Pokemon Showdown clients, from simple chat bots to full GUI applications. The design is opinionated toward clarity and ease of use, prioritizing a clean developer experience over maximum flexibility.

This crate sits between the low-level protocol parsing (handled by an existing protocol crate) and application logic. It manages connection lifecycle, message routing, and client state accumulation.

## Goals

- Simple, idiomatic async Rust API
- Single event loop model—one `recv()` call drives everything
- Automatic state accumulation (challstr, rooms, user info)
- No framework lock-in—works with any async runtime consumer (iced, egui, raw tokio)
- Clear separation from the protocol crate (parsing) and simulator crate (battle logic)

## Non-Goals

- Battle state management (delegated to the simulator crate)
- GUI framework integrations (users wire this themselves)
- Multiple concurrent connections per client instance

## Architecture

```
┌─────────────────────────────────────┐
│         User Application            │
│   (bot, CLI tool, GUI client)       │
├─────────────────────────────────────┤
│      This Crate (ps-client)         │
│  - Connection management            │
│  - State accumulation               │
│  - Send/receive API                 │
├─────────────────────────────────────┤
│        Protocol Crate               │
│  - Message parsing/serialization    │
├─────────────────────────────────────┤
│        Simulator Crate              │
│  - Battle state (optional)          │
└─────────────────────────────────────┘
```

## Core Types

### Client

The main entry point. Owns the connection and accumulated state.

```rust
pub struct Client {
    conn: Connection,
    state: ClientState,
}

impl Client {
    pub async fn connect(url: &str) -> Result<Self>;
    pub async fn login(&self, username: &str, password: &str) -> Result<()>;
    pub async fn next_message(&mut self) -> Option<Message>;

    // Sending
    pub async fn send_chat(&self, room: &RoomId, message: &str) -> Result<()>;
    pub async fn send_pm(&self, user: &str, message: &str) -> Result<()>;
    pub async fn send_choice(&self, room: &RoomId, choice: Choice) -> Result<()>;
    pub async fn search_battle(&self, format: &str) -> Result<()>;
    pub async fn accept_challenge(&self, user: &str) -> Result<()>;
    pub async fn join_room(&self, room: &str) -> Result<()>;
    pub async fn leave_room(&self, room: &RoomId) -> Result<()>;

    // State accessors
    pub fn user(&self) -> Option<&UserInfo>;
    pub fn can_login(&self) -> bool;
    pub fn rooms(&self) -> &HashMap<RoomId, RoomState>;
    pub fn room(&self, id: &RoomId) -> Option<&RoomState>;
}
```

### Connection (Internal)

Channel-based connection using tokio-tungstenite. Spawns two internal tasks for read/write, communicates via bounded mpsc channels.

```rust
struct Connection {
    incoming: mpsc::Receiver<Result<Message>>,
    outgoing: mpsc::Sender<String>,
}
```

This design allows `send_*` methods to take `&self` rather than `&mut self`, and ensures sends never block receives.

### ClientState (Internal)

Accumulated state updated as messages flow through `next_message()`.

```rust
struct ClientState {
    challstr: Option<String>,
    user: Option<UserInfo>,
    rooms: HashMap<RoomId, RoomState>,
}
```

### Public Supporting Types

```rust
pub struct RoomId(pub String);

pub struct UserInfo {
    pub username: String,
    pub logged_in: bool,
}

pub struct RoomState {
    pub id: RoomId,
    pub room_type: RoomType,
    pub users: Vec<String>,
}

pub enum RoomType {
    Chat,
    Battle { format: String },
}

pub enum Choice {
    Move { slot: usize, mega: bool, target: Option<i8> },
    Switch { slot: usize },
    Pass,
}
```

### Message

Re-exported from the protocol crate. The exact variants depend on that crate's design, but conceptually:

```rust
pub enum Message {
    ChallStr(String),
    UpdateUser(UserInfo),
    Chat { room: RoomId, user: String, message: String },
    PM { user: String, message: String },
    Battle { room: RoomId, event: BattleEvent },
    Join { room: RoomId, user: String },
    Leave { room: RoomId, user: String },
    // ... other protocol messages
}
```

## Usage Examples

### Chat Bot

```rust
#[tokio::main]
async fn main() -> Result<()> {
    let mut client = Client::connect("wss://sim.smogon.com/showdown/websocket").await?;

    while let Some(msg) = client.next_message().await {
        let msg = msg?;

        if let Message::ChallStr(_) = &msg {
            client.login("BotName", "password").await?;
        }

        if let Message::Chat { room, user, message } = msg {
            if message.starts_with("!ping") {
                client.send_chat(&room, "pong").await?;
            }
        }
    }

    Ok(())
}
```

### Battle Bot

```rust
use pokemon_sim::Battle;

#[tokio::main]
async fn main() -> Result<()> {
    let mut client = Client::connect("wss://sim.smogon.com/showdown/websocket").await?;
    let mut battles: HashMap<RoomId, Battle> = HashMap::new();

    while let Some(msg) = client.next_message().await {
        let msg = msg?;

        if let Message::ChallStr(_) = &msg {
            client.login("BattleBot", "password").await?;
            client.search_battle("gen9ou").await?;
        }

        if let Message::Battle { room, event } = msg {
            let battle = battles.entry(room.clone()).or_default();
            battle.apply(&event);

            if let BattleEvent::Request(req) = event {
                let choice = decide_move(battle, &req);
                client.send_choice(&room, choice).await?;
            }
        }
    }

    Ok(())
}
```

### GUI Client (iced)

```rust
struct App {
    client: Option<Client>,
    chat_log: Vec<ChatEntry>,
    battles: HashMap<RoomId, BattleView>,
}

enum AppMessage {
    Connected(Result<Client>),
    Pokemon(Result<Message>),
    UserSendChat(RoomId, String),
}

impl Application for App {
    fn update(&mut self, message: AppMessage) -> Command<AppMessage> {
        match message {
            AppMessage::Connected(Ok(client)) => {
                self.client = Some(client);
            }
            AppMessage::Pokemon(Ok(msg)) => {
                match msg {
                    Message::Chat { room, user, message } => {
                        self.chat_log.push(ChatEntry { room, user, message });
                    }
                    Message::Battle { room, event } => {
                        if let Some(view) = self.battles.get_mut(&room) {
                            view.apply(&event);
                        }
                    }
                    _ => {}
                }
            }
            AppMessage::UserSendChat(room, text) => {
                if let Some(client) = &self.client {
                    // Fire and forget, or return a Command
                    let _ = client.send_chat(&room, &text);
                }
            }
            _ => {}
        }
        Command::none()
    }

    fn subscription(&self) -> Subscription<AppMessage> {
        match &self.client {
            Some(client) => {
                // Bridge client.next_message() into iced's subscription system
                iced::subscription::unfold("ps-client", client.clone(), |mut c| async {
                    let msg = c.next_message().await;
                    (msg.map(AppMessage::Pokemon), c)
                })
            }
            None => Subscription::none()
        }
    }
}
```

## Internal Design Decisions

### Channel-Based Connection

The connection spawns two tokio tasks: one for reading from the websocket, one for writing. Communication happens via bounded mpsc channels. This provides:

- True concurrency between send and receive
- `&self` on send methods (no mutable borrow needed)
- Natural backpressure via channel bounds
- Clean shutdown semantics (drop sender = task exits)

### State Accumulation in `next_message()`

When `next_message()` is called, the client:

1. Receives the next message from the incoming channel
2. Updates internal state based on the message type
3. Returns the message to the caller

This ensures state is always consistent with messages the user has seen, with no hidden buffering or race conditions.

### Login Flow

Login does not block waiting for challstr. Instead:

1. User calls `next_message()` in their loop
2. When `Message::ChallStr` arrives, client stores it internally
3. User can then call `login()` which uses the stored challstr
4. If `login()` is called before challstr arrives, it returns an error

This keeps the API simple and the control flow explicit.

## Dependencies

- `tokio` - async runtime
- `tokio-tungstenite` - websocket client
- `futures` - stream utilities
- Protocol crate (existing) - message parsing
- Simulator crate (optional, user-side) - battle state

## Open Questions

1. **Error handling strategy**: Should connection errors be returned from `next_message()` or handled via a separate error channel?

2. **Reconnection**: Should the client handle automatic reconnection, or leave this to the user?

3. **Channel bounds**: What default buffer sizes for the internal channels? Too small risks backpressure in busy rooms, too large risks memory growth.

4. **Room state granularity**: How much room state should we track? Just membership, or also recent chat history?

5. **Authentication methods**: Support for OAuth/token auth in addition to password?

## Future Considerations

- Optional typed subscription helpers if users request them
- Connection state machine exposed via enum for UI status indicators
- Rate limiting helpers for outgoing messages
- Logging/tracing integration
