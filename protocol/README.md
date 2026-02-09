# kazam-protocol

Protocol types and message parsing for the Pokemon Showdown websocket API.

## Overview

This crate provides Rust types and parsers for the Pokemon Showdown protocol, including:
- Server message types (battle events, room messages, user updates, etc.)
- Client message types (commands, chat, battle actions)
- Protocol frame parsing
- Battle request structures

## Status

⚠️ **Work in Progress**: This crate is under active development. APIs may change before 1.0.

## Usage

```rust
use kazam_protocol::{parse_server_frame, ServerMessage};

// Parse a raw protocol message
let frame = parse_server_frame(">battle-gen9ou-12345\n|turn|1")?;
```

## License

MIT
