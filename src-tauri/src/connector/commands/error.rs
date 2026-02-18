use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Serialize)]
pub struct CommandError {
    pub message: String,
    pub kind: String,
}

impl CommandError {
    pub fn new(message: impl Into<String>, kind: impl Into<String>) -> Self {
        CommandError {
            message: message.into(),
            kind: kind.into(),
        }
    }
}

impl From<std::io::Error> for CommandError {
    fn from(err: std::io::Error) -> Self {
        CommandError {
            message: err.to_string(),
            kind: String::from("io"),
        }
    }
}

impl From<serde_json::Error> for CommandError {
    fn from(err: serde_json::Error) -> Self {
        CommandError {
            message: err.to_string(),
            kind: String::from("serialization"),
        }
    }
}

impl From<tauri::Error> for CommandError {
    fn from(err: tauri::Error) -> Self {
        CommandError {
            message: err.to_string(),
            kind: String::from("tauri"),
        }
    }
}

#[derive(Error, Debug)]
pub enum RepositoryError {
    #[error("Connection not found: {0}")]
    NotFound(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Credential error: {0}")]
    Credential(#[from] CredentialError),

    #[error("Migration error from version {from} to {to}")]
    MigrationError { from: u32, to: u32 },
}

#[derive(Debug, Error)]
pub enum CredentialError {
    #[error("Keyring error: {0}")]
    Keyring(#[from] keyring::Error),

    #[error("Task join error: {0}")]
    Join(#[from] tokio::task::JoinError),
}

