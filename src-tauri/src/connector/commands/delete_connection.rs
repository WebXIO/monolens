use tauri::State;

use crate::connector::commands::error::CommandError;
use crate::AppState;

#[tauri::command]
pub async fn delete_connection(state: State<'_, AppState>, id: String) -> Result<(), CommandError> {
    state
        .connection_repository
        .delete(&id)
        .await
        .map_err(|e| CommandError::from_repository(e, "Failed to delete connection."))
}
