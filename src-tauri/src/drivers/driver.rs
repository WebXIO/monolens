use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::errors::DriverError;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TestStage {
    pub status: Option<bool>,
    pub title: String,
}

impl TestStage {
    pub fn new(status: Option<bool>, title: String) -> Self {
        TestStage { status, title }
    }
}

#[async_trait]
pub trait DatabaseDriver: Send + Sync {
    async fn connect(&mut self) -> Result<(), DriverError>;
    async fn disconnect(&mut self) -> Result<(), DriverError>;
    async fn test_connection(&self) -> Result<Vec<TestStage>, DriverError>;
    async fn list_databases(&self) -> Result<Vec<String>, DriverError>;
    async fn list_collections(&self, database_name: &str) -> Result<Vec<String>, DriverError>;
}
