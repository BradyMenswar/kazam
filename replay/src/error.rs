use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ReplayError {
    #[error("failed to read replay log from {path}")]
    ReadLog {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to parse replay log at line {line_number}")]
    ParseLine {
        line_number: usize,
        raw_line: String,
        #[source]
        source: anyhow::Error,
    },

    #[error("message index {index} is out of range 0..={len}")]
    InvalidMessageIndex { index: usize, len: usize },

    #[error("turn {turn} does not exist in this replay")]
    TurnNotFound { turn: u32 },

    #[error("replay speed must be finite and non-negative")]
    InvalidSpeed,
}
