use tauri::State;

use crate::connection_handler::connection_handler::ConnectionHandler;
use crate::connector::models::connection::Connection;
use crate::drivers::errors::DriverError;

#[tauri::command]
pub async fn list_collections(
    connection_handler: State<'_, ConnectionHandler>,
    connection: Connection,
    database_name: String,
) -> Result<Vec<String>, DriverError> {
    let driver_lock = connection_handler
        .get_or_connect(connection)
        .await
        .map_err(|e| DriverError::ConnectionFailed(e.to_string()))?;

    let driver = driver_lock.lock().await;
    driver.list_collections(&database_name).await
}