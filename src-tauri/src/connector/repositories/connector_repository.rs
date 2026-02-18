use std::any::Any;

use async_trait::async_trait;

use crate::connector::commands::error::RepositoryError;
use crate::connector::models::create_connection::CreateConnection;
use crate::connector::models::connection::Connection;
use crate::connector::repositories::storage_wrapper::ConnectionStorage;

#[async_trait]
pub trait ConnectorRepository: Send + Sync {
    fn as_any(&self) -> &dyn Any;
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
    async fn load_storage(&self) -> Result<ConnectionStorage, RepositoryError>;
    async fn write_storage(&self, storage: &ConnectionStorage) -> Result<(), RepositoryError>;
}
