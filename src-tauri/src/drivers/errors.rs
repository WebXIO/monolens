use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error, Serialize)]
pub enum DriverError {
    #[error("Failed to connect: {0}")]
    ConnectionFailed(String),

    #[error("Client not initialized")]
    ClientNotInitialized,

    #[error("Failed to ping database: {0}")]
    PingFailed(String),

    #[error("Failed to list databases: {0}")]
    ListDatabasesFailed(String),

    #[error("Failed to list collections for database {0}: {1}")]
    ListCollectionsFailed(String, String),

    #[error("Failed to get server status: {0}")]
    ServerStatusFailed(String),

    #[error("URI malformed: {0}")]
    UriMalformed(String),

    #[error("Unknown driver type")]
    UnknownDriver,
}
