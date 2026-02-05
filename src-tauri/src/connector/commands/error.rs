use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct CommandError {
    pub message: String,
    pub kind: String,
}

impl CommandError {
    pub fn new(message: impl Into<String>, kind: impl Into<String>) -> Self {
        CommandError {
            message: message.into(),
            kind: kind.into(),
        }
    }
}

impl From<std::io::Error> for CommandError {
    fn from(err: std::io::Error) -> Self {
        CommandError {
            message: err.to_string(),
            kind: String::from("io"),
        }
    }
}

impl From<serde_json::Error> for CommandError {
    fn from(err: serde_json::Error) -> Self {
        CommandError {
            message: err.to_string(),
            kind: String::from("serialization"),
        }
    }
}

impl From<tauri::Error> for CommandError {
    fn from(err: tauri::Error) -> Self {
        CommandError {
            message: err.to_string(),
            kind: String::from("tauri"),
        }
    }
}