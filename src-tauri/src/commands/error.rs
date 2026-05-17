use serde::{Serialize, Serializer};
use thiserror::Error;

/// Errors returned from Tauri commands.
///
/// Implements `Serialize` (via `serde`) so it can cross the IPC boundary; the
/// frontend receives a string for now, but we keep the variant info on the
/// Rust side so error paths stay debuggable in logs.
#[derive(Debug, Error)]
pub enum CommandError {
    #[error("database error: {0}")]
    Database(#[from] anyhow::Error),

    #[error("invalid argument: {0}")]
    InvalidArgument(String),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("hotkey error: {0}")]
    Hotkey(String),

    #[error("clipboard error: {0}")]
    Clipboard(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("tauri error: {0}")]
    Tauri(#[from] tauri::Error),
}

impl Serialize for CommandError {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

pub type CommandResult<T> = Result<T, CommandError>;
