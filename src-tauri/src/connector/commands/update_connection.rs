use tauri::State;

use crate::connection_handler::connection_handler::ConnectionHandler;
use crate::connector::commands::error::CommandError;
use crate::connector::models::connection::Connection;
use crate::AppState;

#[tauri::command]
pub async fn update_connection(
    state: State<'_, AppState>,
    connection_handler: State<'_, ConnectionHandler>,
    id: String,
    connection: Connection,
) -> Result<(), CommandError> {
    state
        .connection_repository
        .update(&id, &connection)
        .await
        .map_err(|e| CommandError::from_repository(e, "Failed to update connection."))?;

    // the next command connects with the new settings
    let _ = connection_handler.disconnect(&id).await;
    Ok(())
}
