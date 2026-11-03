use tauri::State;

use crate::connection_handler::connection_handler::ConnectionHandler;
use crate::drivers::errors::DriverError;
use crate::AppState;

#[tauri::command]
pub async fn list_collections(
    connection_handler: State<'_, ConnectionHandler>,
    app_state: State<'_, AppState>,
    connection_id: String,
    database_name: String,
) -> Result<Vec<String>, DriverError> {
    let driver_lock = connection_handler
        .get_or_connect(app_state.connection_repository.as_ref(), &connection_id)
        .await?;

    let driver = driver_lock.lock().await;
    driver.list_collections(&database_name).await
}
