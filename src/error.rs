use thiserror::Error;

#[derive(Debug, Error)]
pub enum TraceForgeError {
    #[error("I/O error reading {path}: {source}")]
    Io {
        path: String,
        source: std::io::Error,
    },

    #[error("failed to parse JSONL line {line} in {path}: {source}")]
    ParseLine {
        path: String,
        line: usize,
        source: serde_json::Error,
    },

    #[error("empty trace file: {path}")]
    EmptyTrace { path: String },

    #[error("incomplete trace: {path}")]
    IncompleteTrace { path: String },

    #[error("unsupported provider: {0}")]
    UnsupportedProvider(String),

    #[error("output path is inside input directory: {0}")]
    OutputInsideInput(String),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("walkdir error: {0}")]
    Walk(#[from] walkdir::Error),
}

pub type Result<T> = std::result::Result<T, TraceForgeError>;
