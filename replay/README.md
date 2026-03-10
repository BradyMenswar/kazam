# kazam-replay

Replay loading, indexing, and playback controls for Pokemon Showdown logs.

## Overview

`kazam-replay` builds on:

- `kazam-protocol` for parsing replay log lines into `ServerMessage`
- `kazam-battle` for canonical battle-state reduction and snapshots

It provides:

- replay log loading from strings or files
- turn-boundary indexing
- final-state snapshots
- playback controls such as play/pause, seek, next turn, previous turn, and skip to end

## Usage

```rust
use std::time::Duration;

use kazam_replay::{ReplayController, ReplayLog, ReplaySpeed};

let replay = ReplayLog::from_str(
    "|player|p1|Alice|1\n|player|p2|Bob|2\n|turn|1\n|win|Alice"
)?;

let mut controller = ReplayController::new(replay);
controller.first_turn()?;
controller.set_speed(ReplaySpeed::new(8.0)?);
controller.play();
controller.advance_by(Duration::from_millis(500));

assert_eq!(controller.current_turn(), 1);
# Ok::<(), Box<dyn std::error::Error>>(())
```

## Status

Experimental. The playback core is implemented, but UI-facing rendering helpers are expected to live in a separate crate.

## License

MIT
