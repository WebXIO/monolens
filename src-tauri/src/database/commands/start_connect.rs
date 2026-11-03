use crate::{
    connection_handler::connection_handler::ConnectionHandler, drivers::errors::DriverError,
    task_manager::task_manager::TaskManager, AppState,
};
use tauri::{Manager, State};

#[tauri::command]
pub async fn start_connect(
    app_handle: tauri::AppHandle,
    task_manager: State<'_, TaskManager>,
    connection_id: String,
) -> Result<String, DriverError> {
    let id = task_manager
        .spawn_task(async move {
            let app_state = app_handle.state::<AppState>();
            let connection_handler = app_handle.state::<ConnectionHandler>();

            let databases = connection_handler
                .open(app_state.connection_repository.as_ref(), &connection_id)
                .await
                .map_err(|e| serde_json::to_value(&e).unwrap_or_default())?;

            serde_json::to_value(&databases).map_err(|e| {
                serde_json::json!({
                    "kind": "SerializationError",
                    "message": e.to_string(),
                    "user_message": "Failed to serialize database list."
                })
            })
        })
        .await;

    Ok(id)
}
