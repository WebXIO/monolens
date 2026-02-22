use super::task_error::TaskError;
use super::task_manager::TaskManager;
use tauri::State;

#[tauri::command]
pub async fn cancel_task(
    task_manager: State<'_, TaskManager>,
    id: String,
) -> Result<(), TaskError> {
    task_manager.cancel_task(&id).await
}

#[tauri::command]
pub async fn await_task_result(
    task_manager: State<'_, TaskManager>,
    id: String,
) -> Result<serde_json::Value, TaskError> {
    task_manager.await_task(&id).await
}
