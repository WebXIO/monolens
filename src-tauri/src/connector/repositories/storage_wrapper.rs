use std::collections::HashMap;
use serde::{Serialize, Deserialize};

use crate::connector::models::connection::Connection;

/// 1: `uri` held the host, plus `port` and `connectionType`. 2: `target` and `options`.
pub const STORAGE_VERSION: u32 = 2;

#[derive(Serialize, Deserialize, Debug)]
pub struct ConnectionStorage {
    pub version: u32,
    pub connections: HashMap<String, Connection>,
}

impl ConnectionStorage {
    pub fn new() -> Self {
        Self {
            version: STORAGE_VERSION,
            connections: HashMap::new(),
        }
    }
}

impl Default for ConnectionStorage {
    fn default() -> Self {
        Self::new()
    }
}
