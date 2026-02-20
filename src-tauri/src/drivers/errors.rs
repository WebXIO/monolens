use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};
use thiserror::Error;

#[derive(Debug, Error)]
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

impl DriverError {
    /// Returns a human-readable message suitable for display in the frontend.
    pub fn user_message(&self) -> String {
        match self {
            DriverError::ConnectionFailed(msg) => {
                if Self::is_auth_error(msg) {
                    String::from("Authentication failed. Check your username, password, and authentication database.")
                } else {
                    String::from("Failed to connect. Verify your host, port, and that the server is running.")
                }
            }
            DriverError::ClientNotInitialized => {
                String::from("Connection was not initialized. Please try again.")
            }
            DriverError::PingFailed(msg) => {
                if Self::is_auth_error(msg) {
                    String::from("Authentication failed. Check your username, password, and authentication database.")
                } else if Self::is_timeout_error(msg) {
                    String::from("Connection timed out. Check your host, port, and firewall settings.")
                } else {
                    String::from("Could not reach the database. Check your host, port, and firewall settings.")
                }
            }
            DriverError::ListDatabasesFailed(msg) => {
                if Self::is_auth_error(msg) {
                    String::from("Authentication failed. Check your username, password, and permissions.")
                } else {
                    String::from("Failed to retrieve databases. Check your permissions.")
                }
            }
            DriverError::ListCollectionsFailed(_, msg) => {
                if Self::is_auth_error(msg) {
                    String::from("Authentication failed. Check your permissions on this database.")
                } else {
                    String::from("Failed to list collections. Check your permissions on this database.")
                }
            }
            DriverError::ServerStatusFailed(msg) => {
                if Self::is_auth_error(msg) {
                    String::from("Authentication failed. Check your username, password, and permissions.")
                } else {
                    String::from("Could not read server status. Check your permissions.")
                }
            }
            DriverError::UriMalformed(_) => {
                String::from("The connection URI is invalid. Please check the format.")
            }
            DriverError::UnknownDriver => String::from("Unsupported database driver."),
        }
    }

    /// Detects if a raw driver error message indicates an authentication failure.
    fn is_auth_error(msg: &str) -> bool {
        let lower = msg.to_lowercase();
        lower.contains("authentication failed")
            || lower.contains("auth error")
            || lower.contains("unauthorized")
            || lower.contains("autherror")
            || lower.contains("authentication")
    }

    /// Detects if a raw driver error message indicates a timeout.
    fn is_timeout_error(msg: &str) -> bool {
        let lower = msg.to_lowercase();
        lower.contains("timed out")
            || lower.contains("timeout")
            || lower.contains("server selection timeout")
    }

    /// Returns the variant name as a string.
    fn kind(&self) -> &str {
        match self {
            DriverError::ConnectionFailed(_) => "ConnectionFailed",
            DriverError::ClientNotInitialized => "ClientNotInitialized",
            DriverError::PingFailed(_) => "PingFailed",
            DriverError::ListDatabasesFailed(_) => "ListDatabasesFailed",
            DriverError::ListCollectionsFailed(_, _) => "ListCollectionsFailed",
            DriverError::ServerStatusFailed(_) => "ServerStatusFailed",
            DriverError::UriMalformed(_) => "UriMalformed",
            DriverError::UnknownDriver => "UnknownDriver",
        }
    }
}

impl Serialize for DriverError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("DriverError", 3)?;
        state.serialize_field("kind", self.kind())?;
        state.serialize_field("message", &self.to_string())?;
        state.serialize_field("user_message", &self.user_message())?;
        state.end()
    }
}
