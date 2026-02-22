use std::collections::HashMap;
use std::future::Future;
use tokio::sync::Mutex;
use tokio::task::{AbortHandle, JoinHandle};
use super::task_error::TaskError;

pub struct TaskManager {
    join_handles: Mutex<
        HashMap<String, JoinHandle<Result<serde_json::Value, serde_json::Value>>>,
    >,
    abort_handles: Mutex<HashMap<String, AbortHandle>>,
}

impl Default for TaskManager {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskManager {
    pub fn new() -> Self {
        TaskManager {
            join_handles: Mutex::new(HashMap::new()),
            abort_handles: Mutex::new(HashMap::new()),
        }
    }

    pub async fn spawn_task<F>(&self, future: F) -> String
    where
        F: Future<Output = Result<serde_json::Value, serde_json::Value>> + Send + 'static,
    {
        let id = uuid::Uuid::new_v4().to_string();
        let handle = tokio::spawn(future);
        let abort_handle = handle.abort_handle();
        self.join_handles.lock().await.insert(id.clone(), handle);
        self.abort_handles.lock().await.insert(id.clone(), abort_handle);
        log::debug!("TaskManager: spawned task {}", id);
        id
    }

    pub async fn cancel_task(&self, id: &str) -> Result<(), TaskError> {
        let abort_handles = self.abort_handles.lock().await;
        if let Some(abort_handle) = abort_handles.get(id) {
            abort_handle.abort();
            log::debug!("TaskManager: cancelled task {}", id);
            Ok(())
        } else {
            Err(TaskError::NotFound)
        }
    }

    pub async fn await_task(
        &self,
        id: &str,
    ) -> Result<serde_json::Value, TaskError> {
        // Take ownership of JoinHandle to await it.
        // The AbortHandle stays in its map so cancel_task can still abort.
        let handle = {
            let mut join_handles = self.join_handles.lock().await;
            join_handles.remove(id).ok_or(TaskError::NotFound)?
        };

        let result = match handle.await {
            Ok(Ok(value)) => {
                log::debug!("TaskManager: task {} completed successfully", id);
                Ok(value)
            }
            Ok(Err(err_value)) => {
                log::debug!("TaskManager: task {} failed", id);
                Err(TaskError::Failed(err_value))
            }
            Err(join_error) => {
                if join_error.is_cancelled() {
                    log::debug!("TaskManager: task {} was cancelled", id);
                    Err(TaskError::Cancelled)
                } else {
                    log::error!("TaskManager: task {} panicked: {}", id, join_error);
                    Err(TaskError::Failed(serde_json::json!({
                        "kind": "Panic",
                        "message": join_error.to_string(),
                        "user_message": "An unexpected internal error occurred."
                    })))
                }
            }
        };

        // Clean up abort handle after the task is done
        self.abort_handles.lock().await.remove(id);

        result
    }
}
