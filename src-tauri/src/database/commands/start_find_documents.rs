use crate::{
    AppState,
    connection_handler::connection_handler::ConnectionHandler,
    connector::{
        models::{connection::Connection},
        repositories::{
            credentials::hydrate_connection_with_credentials::hydrate_connection_with_credentials,
            file_repository::FileRepository
        }
    },
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

            let repository = &app_state.connection_repository;

            let hydrated_connection =
                if let Some(file_repo) = repository.as_any().downcast_ref::<FileRepository>() {
                    hydrate_connection_with_credentials(file_repo, &connection)
                        .await
                        .map_err(|e| serde_json::to_value(&e).unwrap_or_default())?
                } else {
                    return Err(serde_json::to_value(&DriverError::ConnectionFailed(
                        "Unsupported repository type".into(),
                    ))
                    .unwrap_or_default());
                };

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
