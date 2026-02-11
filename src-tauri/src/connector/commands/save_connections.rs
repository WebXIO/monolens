use tauri::State;

use crate::connector::commands::error::CommandError;
use crate::connector::models::connection::Connection;
use crate::AppState;

#[tauri::command]
pub async fn save_connections(
    state: State<'_, AppState>,
    connections: Vec<Connection>,
) -> Result<(), CommandError> {
    for connection in connections {
        state
            .connection_repository
            .save(connection)
            .await
            .map_err(|e| CommandError::new(e.to_string(), "repository"))?;
    }
    Ok(())
}
