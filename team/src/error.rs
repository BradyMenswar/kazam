use thiserror::Error;

#[derive(Debug, Error)]
pub enum TeamError {
    #[error("failed to parse team JSON")]
    Json(#[from] serde_json::Error),

    #[error("invalid packed team data")]
    InvalidPacked,

    #[error("invalid export format at line {line_number}: {message}")]
    InvalidExport { line_number: usize, message: String },

    #[error("unknown stat name `{0}`")]
    UnknownStat(String),
}
