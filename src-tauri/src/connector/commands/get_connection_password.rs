use tauri::State;

use crate::connector::commands::error::CommandError;
use crate::AppState;

#[tauri::command]
pub async fn get_connection_password(
    state: State<'_, AppState>,
    id: String,
) -> Result<Option<String>, CommandError> {
    // the edit dialog shows an empty field when the password can't be read
    Ok(state
        .connection_repository
        .get_password(&id)
        .await
        .unwrap_or(None))
}
