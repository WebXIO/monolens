use async_trait::async_trait;

use crate::connector::commands::error::RepositoryError;
use crate::connector::models::create_connection::CreateConnection;
use crate::connector::models::connection::Connection;

#[async_trait]
pub trait ConnectorRepository: Send + Sync {
    async fn list(&self) -> Result<Vec<Connection>, RepositoryError>;
    async fn get(&self, id: &str) -> Result<Connection, RepositoryError>;
    async fn save(
        &self,
        connection: CreateConnection,
    ) -> Result<Connection, RepositoryError>;
    async fn update(
        &self,
        id: &str,
        connection: &Connection,
    ) -> Result<(), RepositoryError>;
    async fn delete(&self, id: &str) -> Result<(), RepositoryError>;
    /// The stored password, `None` when there is none.
    async fn get_password(&self, id: &str) -> Result<Option<String>, RepositoryError>;
    /// The connection with its password filled in when it uses authentication.
    async fn hydrate(&self, connection: &Connection) -> Result<Connection, RepositoryError>;
}
