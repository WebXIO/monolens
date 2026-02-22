use crate::{
    connector::models::connection::Connection,
    drivers::{driver::TestStage, errors::DriverError, factory::DatabaseDriverFactory},
    task_manager::task_manager::TaskManager,
};
use serde::Serialize;
use tauri::{Emitter, State};

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TestProgressEvent {
    pub task_id: String,
    pub stage_index: usize,
    pub stage: TestStage,
}

#[tauri::command]
pub async fn start_test_connection(
    app_handle: tauri::AppHandle,
    task_manager: State<'_, TaskManager>,
    connection: Connection,
) -> Result<String, DriverError> {
    let id = task_manager
        .spawn_task(async move {
            let mut driver = DatabaseDriverFactory::create(connection)
                .map_err(|e| serde_json::to_value(&e).unwrap_or_default())?;

            driver
                .connect()
                .await
                .map_err(|e| serde_json::to_value(&e).unwrap_or_default())?;

            let result = driver
                .test_connection(Box::new({
                    let app = app_handle.clone();
                    move |index, stage| {
                        // We don't have the task_id inside here, but the frontend
                        // subscribes right before starting, so it captures all events.
                        // We'll use an empty task_id — the frontend ignores it.
                        let _ = app.emit(
                            "test-connection-progress",
                            TestProgressEvent {
                                task_id: String::new(),
                                stage_index: index,
                                stage: stage.clone(),
                            },
                        );
                    }
                }))
                .await
                .map_err(|e| serde_json::to_value(&e).unwrap_or_default())?;

            serde_json::to_value(&result).map_err(|e| {
                serde_json::json!({
                    "kind": "SerializationError",
                    "message": e.to_string(),
                    "user_message": "Failed to serialize test results."
                })
            })
        })
        .await;

    Ok(id)
}
