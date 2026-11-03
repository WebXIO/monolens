use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use super::errors::DriverError;
use crate::database::models::find_document::FindDocumentsResult;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum StageStatus {
    Pending,
    Passed,
    Failed,
    /// the check could not run, e.g. without the privilege; the test goes on
    Skipped,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct TestStage {
    pub status: StageStatus,
    pub title: String,
    /// e.g. the detected version, or why a stage was skipped
    pub detail: Option<String>,
}

impl TestStage {
    pub fn pending(title: &str) -> Self {
        TestStage {
            status: StageStatus::Pending,
            title: title.to_string(),
            detail: None,
        }
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
