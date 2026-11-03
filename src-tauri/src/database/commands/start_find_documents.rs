use crate::{
    AppState,
    connection_handler::connection_handler::ConnectionHandler,
    drivers::errors::DriverError, task_manager::task_manager::TaskManager
};
use tauri::{Manager, State};

#[tauri::command]
pub async fn start_find_documents(
    app_handle: tauri::AppHandle,
    task_manager: State<'_, TaskManager>,
    connection_id: String,
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

            let driver_lock = connection_handler
                .get_or_connect(app_state.connection_repository.as_ref(), &connection_id)
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
