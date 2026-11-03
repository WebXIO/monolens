use crate::{
    connector::models::{connection::Connection, normalize::normalize},
    drivers::{driver::TestStage, errors::DriverError, factory::DatabaseDriverFactory},
    task_manager::task_manager::TaskManager,
};
use serde::Serialize;
use tauri::{Emitter, State};

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TestProgressEvent {
    /// the id the ui chose for this test, so its listener ignores other tests
    pub test_id: String,
    pub stage_index: usize,
    pub stage: TestStage,
}

/// The form data checked like on save, so credentials in a connection string are used the same way.
fn prepared(connection: Connection) -> Result<Connection, DriverError> {
    let (target, authentication) = normalize(
        &connection.name,
        connection.target.clone(),
        connection.authentication.clone(),
    )
    .map_err(DriverError::InvalidConnection)?;
    Ok(Connection { target, authentication, ..connection })
}

#[tauri::command]
pub async fn start_test_connection(
    app_handle: tauri::AppHandle,
    task_manager: State<'_, TaskManager>,
    connection: Connection,
    test_id: String,
) -> Result<String, DriverError> {
    let id = task_manager
        .spawn_task(async move {
            let connection =
                prepared(connection).map_err(|e| serde_json::to_value(&e).unwrap_or_default())?;
            let mut driver = DatabaseDriverFactory::create(connection)
                .map_err(|e| serde_json::to_value(&e).unwrap_or_default())?;

            driver
                .connect()
                .await
                .map_err(|e| serde_json::to_value(&e).unwrap_or_default())?;

            let result = driver
                .test_connection(Box::new({
                    let app = app_handle.clone();
                    let test_id = test_id.clone();
                    move |index, stage| {
                        let _ = app.emit(
                            "test-connection-progress",
                            TestProgressEvent {
                                test_id: test_id.clone(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connector::models::{
        authentication::{AuthenticationKind, AuthenticationOptions},
        connection::{ConnectionTarget, ConnectorKind},
        options::ConnectionOptions,
    };

    fn form(target: ConnectionTarget) -> Connection {
        Connection {
            id: String::new(),
            name: "test".into(),
            connector: ConnectorKind::MongoDb,
            target,
            authentication: AuthenticationOptions::default(),
            options: ConnectionOptions::default(),
        }
    }

    #[test]
    fn credentials_in_a_connection_string_are_used_like_on_save() {
        let connection = prepared(form(ConnectionTarget::Uri { uri: "mongodb://ada:pw@h/".into() })).unwrap();

        assert_eq!(connection.target, ConnectionTarget::Uri { uri: "mongodb://h/".into() });
        assert_eq!(connection.authentication.kind, AuthenticationKind::BASIC);
        assert_eq!(connection.authentication.username.as_deref(), Some("ada"));
        assert_eq!(connection.authentication.password.as_deref(), Some("pw"));
    }

    #[test]
    fn an_invalid_connection_says_why() {
        let err = prepared(form(ConnectionTarget::Host { host: "h:1".into(), port: 27017 })).unwrap_err();
        assert!(matches!(&err, DriverError::InvalidConnection(_)));
        assert!(err.user_message().contains("port field"));
    }
}
