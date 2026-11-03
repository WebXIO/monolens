use tauri::State;

use crate::connection_handler::connection_handler::ConnectionHandler;
use crate::connector::commands::error::CommandError;
use crate::AppState;

#[tauri::command]
pub async fn delete_connection(
    state: State<'_, AppState>,
    connection_handler: State<'_, ConnectionHandler>,
    id: String,
) -> Result<(), CommandError> {
    let _ = connection_handler.disconnect(&id).await;

    state
        .connection_repository
        .delete(&id)
        .await
        .map_err(|e| CommandError::from_repository(e, "Failed to delete connection."))
}
