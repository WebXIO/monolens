use tauri::State;

use crate::connector::commands::error::CommandError;
use crate::connector::models::connection::Connection;
use crate::AppState;

#[tauri::command]
pub async fn get_connection(
    state: State<'_, AppState>,
    id: String,
) -> Result<Connection, CommandError> {
    state
        .connection_repository
        .get(&id)
        .await
        .map_err(|e| CommandError::new(e.to_string(), "repository", "Failed to load connection."))
}
