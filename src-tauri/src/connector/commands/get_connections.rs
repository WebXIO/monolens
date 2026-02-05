use std::fs;
use tauri::{AppHandle, Manager};

use crate::connector::commands::error::CommandError;
use crate::connector::models::connection::Connection;

const CONNECTIONS_FILE: &str = "connections.json";

#[tauri::command]
pub fn get_connections(app: AppHandle) -> Result<Vec<Connection>, CommandError> {
    let app_config_dir = app.path().app_config_dir()?;
    let connections_path = app_config_dir.join(CONNECTIONS_FILE);

    // Return empty vec if file doesn't exist yet
    if !connections_path.exists() {
        return Ok(Vec::new());
    }

    // Read and parse the file
    let content = fs::read_to_string(&connections_path)?;
    let connections: Vec<Connection> = serde_json::from_str(&content)?;

    Ok(connections)
}