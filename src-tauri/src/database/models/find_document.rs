use serde::{Deserialize, Serialize};

/// One page of a find, as the query tab asks for it.
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FindQuery {
    pub database_name: String,
    pub collection_name: String,
    /// extended JSON objects; null or {} means none
    #[serde(default)]
    pub filter: serde_json::Value,
    #[serde(default)]
    pub sort: serde_json::Value,
    #[serde(default)]
    pub projection: serde_json::Value,
    #[serde(default)]
    pub skip: u64,
    pub limit: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FindDocumentsResult {
    pub documents: Vec<serde_json::Value>,
    pub total_count: u64,
    /// an empty filter is counted from the collection's metadata, which is fast but may be off
    #[serde(default)]
    pub total_count_estimated: bool,
}
