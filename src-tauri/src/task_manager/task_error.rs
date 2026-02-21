use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};

#[derive(Debug)]
pub enum TaskError {
    NotFound,
    Cancelled,
    Failed(serde_json::Value),
}

impl Serialize for TaskError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("TaskError", 3)?;
        match self {
            TaskError::NotFound => {
                state.serialize_field("kind", "NotFound")?;
                state.serialize_field("message", "Task not found")?;
                state.serialize_field("user_message", "The requested operation was not found.")?;
            }
            TaskError::Cancelled => {
                state.serialize_field("kind", "Cancelled")?;
                state.serialize_field("message", "Task was cancelled")?;
                state.serialize_field("user_message", "The operation was cancelled.")?;
            }
            TaskError::Failed(value) => {
                state.serialize_field("kind", "Failed")?;
                state.serialize_field("message", &value.to_string())?;
                state.serialize_field(
                    "user_message",
                    "The operation failed. Check details for more information.",
                )?;
            }
        }
        state.end()
    }
}