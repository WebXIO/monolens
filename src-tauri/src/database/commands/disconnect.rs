use tauri::State;

use crate::connection_handler::connection_handler::ConnectionHandler;
use crate::drivers::errors::DriverError;

/// Drops the connected driver; the next command builds it from the saved settings again.
#[tauri::command]
pub async fn disconnect(
    connection_handler: State<'_, ConnectionHandler>,
    connection_id: String,
) -> Result<(), DriverError> {
    connection_handler.disconnect(&connection_id).await
}
