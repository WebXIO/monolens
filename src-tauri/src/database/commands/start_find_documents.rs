use crate::{
    AppState,
    connection_handler::connection_handler::ConnectionHandler,
    connector::models::connection::Connection,
    drivers::{driver::DatabaseDriver, errors::DriverError}, task_manager::task_manager::TaskManager
};
use std::sync::Arc;
use tauri::{Manager, State};
use tokio::sync::Mutex;

#[tauri::command]
pub async fn start_find_documents(
    app_handle: tauri::AppHandle,
    task_manager: State<'_, TaskManager>,
    connection: Connection,
    database_name: String,
    collection_name: String,
    filter: serde_json::Value,
    skip: u64,
    limit: i64
) -> Result<String, DriverError> {
    let id = task_manager
        .spawn_task(async move {
            let app_state = app_handle.state::<AppState>();
            let connection_handler = app_handle.state::<ConnectionHandler>();

            let hydrated_connection = app_state
                .connection_repository
                .hydrate(&connection)
                .await
                .map_err(|_| {
                    serde_json::to_value(&DriverError::ConnectionFailed(
                        "Could not fetch credentials".into(),
                    ))
                    .unwrap_or_default()
                })?;

            let driver_lock: Arc<Mutex<Box<dyn DatabaseDriver>>> = connection_handler
                .get_or_connect(hydrated_connection)
                .await
                .map_err(|e| serde_json::to_value(&e).unwrap_or_default())?;

            let driver = driver_lock.lock().await;
            let documents = driver
                .find_documents(&database_name, &collection_name, filter, skip, limit)
                .await
                .map_err(|e| serde_json::to_value(&e).unwrap_or_default())?;

            serde_json::to_value(&documents).map_err(|e| {
                serde_json::json!({
                    "kind": "SerializationError",
                    "message": e.to_string(),
                    "user_message": "Failed to serialize document list."
                })
            })
        })
        .await;

    Ok(id)
}
