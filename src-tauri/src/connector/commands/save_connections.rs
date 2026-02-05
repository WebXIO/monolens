use std::fs;
use tauri::{AppHandle, Manager};

use crate::connector::commands::error::CommandError;
use crate::connector::models::connection::Connection;

const CONNECTIONS_FILE: &str = "connections.json";

#[tauri::command]
pub fn save_connections(app: AppHandle, connections: Vec<Connection>) -> Result<(), CommandError> {
    let app_config_dir = app.path().app_config_dir()?;

    // Create directory if it doesn't exist
    if !app_config_dir.exists() {
        fs::create_dir_all(&app_config_dir)?;
    }

    let connections_path = app_config_dir.join(CONNECTIONS_FILE);

    // Serialize and write connections
    let content = serde_json::to_string_pretty(&connections)?;
    fs::write(&connections_path, content)?;

    Ok(())
}
