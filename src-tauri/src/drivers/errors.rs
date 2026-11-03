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

    #[error("Failed to find documents: {0}")]
    FindDocumentsFailed(String),

    #[error("Invalid query: {0}")]
    InvalidQuery(String),

    #[error("Invalid connection: {0}")]
    InvalidConnection(String),

    #[error("Connection not found: {0}")]
    ConnectionNotFound(String),

    #[error("Could not read the saved connections: {0}")]
    StorageFailed(String),

    #[error("Could not read the password from the keyring: {0}")]
    CredentialsUnavailable(String),
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
            DriverError::FindDocumentsFailed(msg) => {
                if Self::is_auth_error(msg) || msg.to_lowercase().contains("not authorized") {
                    String::from("Not allowed to read this collection. Check your permissions on this database.")
                } else {
                    // the server's message names the mistake in the query
                    format!("The query failed: {msg}")
                }
            }
            DriverError::InvalidQuery(msg) | DriverError::InvalidConnection(msg) => msg.clone(),
            DriverError::ConnectionNotFound(_) => {
                String::from("This connection no longer exists.")
            }
            DriverError::StorageFailed(_) => {
                String::from("Could not read the saved connections.")
            }
            DriverError::CredentialsUnavailable(_) => {
                String::from("The password of this connection could not be read from the system keyring. Edit the connection and enter it again.")
            }
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
            DriverError::FindDocumentsFailed(_) => "FindDocumentsFailed",
            DriverError::InvalidQuery(_) => "InvalidQuery",
            DriverError::InvalidConnection(_) => "InvalidConnection",
            DriverError::ConnectionNotFound(_) => "ConnectionNotFound",
            DriverError::StorageFailed(_) => "StorageFailed",
            DriverError::CredentialsUnavailable(_) => "CredentialsUnavailable",
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognises_authentication_failures_in_driver_messages() {
        for err in [
            DriverError::ConnectionFailed("SCRAM failure: Authentication failed.".into()),
            DriverError::PingFailed("Command failed: Unauthorized".into()),
            DriverError::ListDatabasesFailed("AuthError: bad auth".into()),
            DriverError::ServerStatusFailed("authentication failed".into()),
        ] {
            assert!(err.user_message().starts_with("Authentication failed"), "{err:?}");
        }
    }

    #[test]
    fn recognises_timeouts_when_pinging() {
        let err = DriverError::PingFailed("Server selection timeout: No available servers".into());
        assert!(err.user_message().starts_with("Connection timed out"));
    }

    #[test]
    fn falls_back_to_a_generic_message() {
        assert!(DriverError::ConnectionFailed("dns error".into()).user_message().starts_with("Failed to connect"));
        assert!(DriverError::PingFailed("connection refused".into()).user_message().starts_with("Could not reach"));
    }

    #[test]
    fn credential_and_storage_failures_have_their_own_messages() {
        assert!(DriverError::CredentialsUnavailable("locked".into()).user_message().contains("keyring"));
        assert_eq!(DriverError::ConnectionNotFound("x".into()).user_message(), "This connection no longer exists.");
        assert_eq!(DriverError::StorageFailed("io".into()).user_message(), "Could not read the saved connections.");
    }

    #[test]
    fn query_errors_pass_the_server_message_on() {
        assert_eq!(
            DriverError::FindDocumentsFailed("unknown top level operator: $foo".into()).user_message(),
            "The query failed: unknown top level operator: $foo"
        );
        assert!(DriverError::FindDocumentsFailed("not authorized on shop to execute command".into())
            .user_message()
            .starts_with("Not allowed"));
        assert_eq!(DriverError::InvalidQuery("The sort must be an object".into()).user_message(), "The sort must be an object");
    }

    #[test]
    fn serializes_kind_message_and_user_message() {
        let value = serde_json::to_value(DriverError::CredentialsUnavailable("locked".into())).unwrap();
        assert_eq!(value["kind"], "CredentialsUnavailable");
        assert_eq!(value["message"], "Could not read the password from the keyring: locked");
        assert!(value["user_message"].as_str().unwrap().contains("keyring"));
    }
}
