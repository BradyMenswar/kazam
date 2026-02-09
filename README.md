# Kazam

A Rust library for interacting with Pokemon Showdown servers.

## Overview

Kazam provides a complete toolkit for building Pokemon Showdown bots and clients in Rust. The project is split into three crates that handle different aspects of the Pokemon Showdown protocol and battle simulation.

## Crates

### kazam-protocol

Protocol types and message parsing for the Pokemon Showdown websocket API.

This crate provides all the type definitions and parsers needed to communicate with Pokemon Showdown servers. It handles parsing the custom protocol format into strongly-typed Rust structs.

**Status:** Published on crates.io

```toml
[dependencies]
kazam-protocol = "0.1.0"
```

### kazam-battle

Battle state tracking and domain types for Pokemon Showdown battles.

This crate provides structures and logic for tracking battle state, including Pokemon stats, active battle conditions, move tracking, and team composition. It's designed to maintain an accurate representation of battle state as events occur.

**Status:** Work in progress - API may change

```toml
[dependencies]
kazam-battle = "0.1.0"
```

### kazam-client

Async websocket client for Pokemon Showdown with an event-driven handler interface.

The client crate provides a high-level async interface for connecting to Pokemon Showdown servers. It features automatic reconnection, room state tracking, and a trait-based handler system for responding to server events.

**Status:** Work in progress - API may change

```toml
[dependencies]
kazam-client = "0.1.0"
```

## Quick Start

Here's a minimal example of connecting to Pokemon Showdown and responding to events:

```rust
use kazam_client::{KazamClient, KazamHandler, SHOWDOWN_URL};
use anyhow::Result;

#[derive(Default)]
struct MyBot;

#[async_trait::async_trait]
impl KazamHandler for MyBot {
    async fn on_challstr(&mut self, challstr: &str) {
        println!("Connected to server");
    }

    async fn on_logged_in(&mut self, username: &str) {
        println!("Logged in as {}", username);
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut client = KazamClient::connect(SHOWDOWN_URL).await?;
    let mut handler = MyBot::default();

    client.run(&mut handler).await?;
    Ok(())
}
```

## Development Status

This project is under active development. All crates are currently pre-1.0, which means breaking changes may occur between minor versions. The protocol crate is the most stable, while the battle and client crates are still evolving.

## Contributing

Contributions are welcome. Please open an issue to discuss significant changes before submitting a pull request.

## License

This project is licensed under the MIT License. See the LICENSE file for details.
