# kazam-client

Async websocket client for Pokemon Showdown with an event-driven handler interface.

## Overview

This crate provides a high-level async client for connecting to Pokemon Showdown servers. It features:
- Automatic websocket connection management with reconnection support
- Event-driven handler trait for processing server messages
- Room and battle state tracking
- Type-safe command sending via handles

## Status

⚠️ **Work in Progress**: This crate is under active development. APIs may change before 1.0.

## Usage

```rust
use kazam_client::{KazamClient, KazamHandler, SHOWDOWN_URL};

#[derive(Default)]
struct MyHandler;

#[async_trait::async_trait]
impl KazamHandler for MyHandler {
    async fn on_challstr(&mut self, challstr: &str) {
        println!("Received challstr: {}", challstr);
    }

    async fn on_logged_in(&mut self, username: &str) {
        println!("Logged in as {}", username);
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut client = KazamClient::connect(SHOWDOWN_URL).await?;
    let handle = client.handle();

    let mut handler = MyHandler::default();
    client.run(&mut handler).await?;

    Ok(())
}
```

## License

MIT
