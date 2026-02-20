use tauri::State;

use crate::connector::commands::error::CommandError;
use crate::connector::models::connection::Connection;
use crate::AppState;
use crate::connector::models::create_connection::CreateConnection;

#[tauri::command]
pub async fn create_connection(
    state: State<'_, AppState>,
    connection: CreateConnection,
) -> Result<Connection, CommandError> {
    state
        .connection_repository
        .save(connection)
        .await
        .map_err(|e| CommandError::new(e.to_string(), "repository", "Failed to create connection."))
}
