use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FindDocumentsResult {
    pub documents: Vec<serde_json::Value>,
    #[serde(rename = "totalCount")]
    pub total_count: u64,
}