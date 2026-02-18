use std::collections::HashMap;
use serde::{Serialize, Deserialize};

use crate::connector::{ models::connection::Connection};

pub const STORAGE_VERSION: u32 = 1;

#[derive(Serialize, Deserialize)]
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
