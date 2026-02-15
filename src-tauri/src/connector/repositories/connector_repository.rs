use std::error::Error;

use async_trait::async_trait;

use crate::connector::models::create_connection::CreateConnection;
use crate::connector::models::connection::Connection;

#[async_trait]
pub trait ConnectorRepository: Send + Sync {
    async fn list(&self) -> Result<Vec<Connection>, Box<dyn Error + Send + Sync>>;
    async fn get(&self, id: &str) -> Result<Connection, Box<dyn Error + Send + Sync>>;
    async fn save(
        &self,
        connection: CreateConnection,
    ) -> Result<Connection, Box<dyn Error + Send + Sync>>;
    async fn update(
        &self,
        id: &str,
        connection: &Connection,
    ) -> Result<(), Box<dyn Error + Send + Sync>>;
    async fn delete(&self, id: &str) -> Result<(), Box<dyn Error + Send + Sync>>;
}
