use tauri::State;

use crate::connector::commands::error::CommandError;
use crate::connector::models::connection::Connection;
use crate::AppState;

#[tauri::command]
pub async fn update_connection(
    state: State<'_, AppState>,
    id: String,
    connection: Connection,
) -> Result<(), CommandError> {
    state
        .connection_repository
        .update(&id, &connection)
        .await
        .map_err(|e| CommandError::new(e.to_string(), "repository", "Failed to update connection."))
}
