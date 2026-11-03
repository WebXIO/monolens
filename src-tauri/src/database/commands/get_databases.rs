use crate::AppState;
use crate::connection_handler::connection_handler::ConnectionHandler;
use crate::connector::models::connection::Connection;
use crate::drivers::errors::DriverError;
use tauri::State;

#[tauri::command]
pub async fn get_databases(
    connection_handler: State<'_, ConnectionHandler>,
    app_state: State<'_, AppState>,
    connection: Connection,
) -> Result<Vec<String>, DriverError> {
    let hydrated_connection = app_state
        .connection_repository
        .hydrate(&connection)
        .await
        .map_err(|_| DriverError::ConnectionFailed("Could not fetch credentials".into()))?;

    let driver_lock = connection_handler
        .get_or_connect(hydrated_connection)
        .await?;

    let driver = driver_lock.lock().await;
    driver.list_databases().await
}
