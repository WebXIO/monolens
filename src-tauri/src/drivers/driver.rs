use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use super::errors::DriverError;
use crate::database::models::find_document::FindDocumentsResult;

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

pub type ProgressCallback = Box<dyn Fn(usize, &TestStage) + Send + Sync>;

#[async_trait]
pub trait DatabaseDriver: Send + Sync {
    async fn connect(&mut self) -> Result<(), DriverError>;
    async fn disconnect(&mut self) -> Result<(), DriverError>;
    async fn test_connection(
        &self,
        on_progress: ProgressCallback,
    ) -> Result<Vec<TestStage>, DriverError>;
    async fn list_databases(&self) -> Result<Vec<String>, DriverError>;
    async fn list_collections(&self, database_name: &str) -> Result<Vec<String>, DriverError>;
    async fn find_documents(
        &self,
        database_name: &str,
        collection_name: &str,
        filter: serde_json::Value,
        skip: u64,
        limit: i64
    ) -> Result<FindDocumentsResult, DriverError>;
}
