use tauri::State;

use crate::connector::commands::error::CommandError;
use crate::connector::models::connection::Connection;
use crate::AppState;

#[tauri::command]
pub async fn get_connections(state: State<'_, AppState>) -> Result<Vec<Connection>, CommandError> {
    state
        .connection_repository
        .list()
        .await
        .map_err(|e| CommandError::new(e.to_string(), "repository", "Failed to load connections."))
}
