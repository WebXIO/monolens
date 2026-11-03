use crate::{
    AppState,
    connection_handler::connection_handler::ConnectionHandler,
    database::models::find_document::FindQuery,
    drivers::errors::DriverError, task_manager::task_manager::TaskManager
};
use tauri::{Manager, State};

#[tauri::command]
pub async fn start_find_documents(
    app_handle: tauri::AppHandle,
    task_manager: State<'_, TaskManager>,
    connection_id: String,
    query: FindQuery,
) -> Result<String, DriverError> {
    let id = task_manager
        .spawn_task(async move {
            let app_state = app_handle.state::<AppState>();
            let connection_handler = app_handle.state::<ConnectionHandler>();

            let driver = connection_handler
                .get_or_connect(app_state.connection_repository.as_ref(), &connection_id)
                .await
                .map_err(|e| serde_json::to_value(&e).unwrap_or_default())?;

            let documents = driver
                .find_documents(&query)
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
