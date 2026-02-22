use crate::AppState;
use crate::connection_handler::connection_handler::ConnectionHandler;
use crate::connector::repositories::credentials::hydrate_connection_with_credentials::hydrate_connection_with_credentials;
use crate::connector::repositories::file_repository::FileRepository;
use crate::connector::models::connection::Connection;
use crate::drivers::errors::DriverError;
use tauri::State;

#[tauri::command]
pub async fn get_databases(
    connection_handler: State<'_, ConnectionHandler>,
    app_state: State<'_, AppState>,
    connection: Connection,
) -> Result<Vec<String>, DriverError> {
    // get repository from AppState
    let repository = &app_state.connection_repository;

    // hydrate_connection_with_credentials needs a concrete type
    let hydrated_connection = if let Some(file_repo) = repository.as_any().downcast_ref::<FileRepository>() {
        hydrate_connection_with_credentials(file_repo, &connection).await?
    } else {
        return Err(DriverError::ConnectionFailed("Unsupported repository type".into()));
    };

    let driver_lock = connection_handler
        .get_or_connect(hydrated_connection)
        .await?;

    let driver = driver_lock.lock().await;
    driver.list_databases().await
}
