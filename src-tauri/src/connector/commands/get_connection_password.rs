use tauri::State;

use crate::connector::commands::error::CommandError;
use crate::connector::repositories::file_repository::FileRepository;
use crate::AppState;

#[tauri::command]
pub async fn get_connection_password(
    state: State<'_, AppState>,
    id: String,
) -> Result<Option<String>, CommandError> {
    let repository = &state.connection_repository;

    let file_repo = repository
        .as_any()
        .downcast_ref::<FileRepository>()
        .ok_or_else(|| {
            CommandError::new(
                "Unsupported repository type",
                "repository",
                "Could not retrieve password.",
            )
        })?;

    match file_repo.credential_service.get_password(&id).await {
        Ok(password) => Ok(Some(password)),
        Err(_) => Ok(None),
    }
}
