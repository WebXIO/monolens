use crate::{connector::{models::{authentication::AuthenticationKind, connection::Connection}, repositories::file_repository::FileRepository}, drivers::errors::DriverError};

pub async fn hydrate_connection_with_credentials(
    repository: &FileRepository,
    connection: &Connection,
) -> Result<Connection, DriverError> {
    let mut c = connection.clone();
    if let AuthenticationKind::BASIC = c.authentication.kind {
        let password = repository
            .credential_service
            .get_password(&c.id)
            .await
            .map_err(|_| DriverError::ConnectionFailed("Could not fetch credentials".into()))?;
        c.authentication.password = Some(password);
    }
    Ok(c)
}