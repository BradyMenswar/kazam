# kazam-team

Pokemon Showdown team formats and conversion utilities.

## Overview

`kazam-team` provides a canonical Rust model for teams and codecs for the three core Showdown team formats:

- export format
- JSON format
- packed format

The crate exposes:

- `Teams::unpack`
- `Teams::pack`
- `Teams::import`
- `Teams::export`
- `Teams::export_set`

## Usage

```rust
use kazam_team::Teams;

let packed = "Articuno||leftovers|pressure|icebeam,hurricane,substitute,roost|Modest|252,,,252,4,||,,,30,30,|||";

let team = Teams::unpack(packed)?;
let exported = Teams::export(&team);
let reparsed = Teams::import(&exported)?;

assert_eq!(team, reparsed);
# Ok::<(), Box<dyn std::error::Error>>(())
```

## Status

Experimental. Conversion between core team formats is implemented. Validation and random team generation are intentionally out of scope for this crate.

## License

MIT
