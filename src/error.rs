use std::io;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0}")]
    User(String),

    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("TOML parse error: {0}")]
    Toml(#[from] toml::de::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("command failed: {command}\n{stderr}")]
    Command { command: String, stderr: String },
}

pub type Result<T> = std::result::Result<T, Error>;
