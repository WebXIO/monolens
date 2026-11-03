use serde_json::Value;

use crate::connector::commands::error::RepositoryError;
use crate::connector::models::{
    authentication::{AuthenticationKind, AuthenticationOptions},
    connection::{Connection, ConnectionTarget, ConnectorKind},
    options::ConnectionOptions,
    uri::{is_connection_string, split_uri_credentials},
};
use crate::connector::repositories::storage_wrapper::{ConnectionStorage, STORAGE_VERSION};

/// A password found in a version 1 connection string; it moves to the credential store.
#[derive(Debug, PartialEq)]
pub struct MovedPassword {
    pub connection_id: String,
    pub password: String,
}

/// Version 1 kept the host in `uri` next to `port`, and a `connectionType` nothing read.
/// A `uri` that is a connection string becomes a `Uri` target without its credentials.
pub fn migrate_v1(value: &Value) -> Result<(ConnectionStorage, Vec<MovedPassword>), RepositoryError> {
    let invalid = || RepositoryError::MigrationError { from: 1, to: STORAGE_VERSION };

    let connections = value
        .get("connections")
        .and_then(Value::as_object)
        .ok_or_else(invalid)?;

    let mut storage = ConnectionStorage::new();
    let mut moved = Vec::new();

    for (key, old) in connections {
        let id = old.get("id").and_then(Value::as_str).unwrap_or(key).to_string();
        let name = old.get("name").and_then(Value::as_str).ok_or_else(invalid)?.to_string();
        let uri = old.get("uri").and_then(Value::as_str).ok_or_else(invalid)?.trim();
        let port = old
            .get("port")
            .and_then(Value::as_u64)
            .and_then(|port| u16::try_from(port).ok())
            .unwrap_or(27017);

        let mut authentication: AuthenticationOptions = match old.get("authentication") {
            Some(auth) => serde_json::from_value(auth.clone()).map_err(|_| invalid())?,
            None => AuthenticationOptions::default(),
        };
        authentication.password = None;

        let target = if is_connection_string(uri) {
            let split = split_uri_credentials(uri).map_err(|_| invalid())?;
            if split.username.is_some() || split.password.is_some() {
                authentication.kind = AuthenticationKind::BASIC;
            }
            if authentication.username.as_deref().unwrap_or("").is_empty() {
                authentication.username = split.username;
            }
            if let Some(password) = split.password {
                moved.push(MovedPassword { connection_id: id.clone(), password });
            }
            ConnectionTarget::Uri { uri: split.uri }
        } else {
            ConnectionTarget::Host { host: uri.to_string(), port }
        };

        storage.connections.insert(
            id.clone(),
            Connection {
                id,
                name,
                connector: ConnectorKind::MongoDb,
                target,
                authentication,
                options: ConnectionOptions::default(),
            },
        );
    }

    Ok((storage, moved))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn v1(connections: Value) -> Value {
        json!({ "version": 1, "connections": connections })
    }

    #[test]
    fn turns_host_and_port_into_a_host_target() {
        let (storage, moved) = migrate_v1(&v1(json!({
            "a": {
                "id": "a", "name": "Local", "uri": "localhost", "port": 27018,
                "connector": "MongoDb", "connectionType": "Standalone",
                "authentication": { "kind": "BASIC", "username": "root", "password": null, "database": "admin" }
            }
        })))
        .unwrap();

        let connection = &storage.connections["a"];
        assert_eq!(storage.version, STORAGE_VERSION);
        assert_eq!(connection.name, "Local");
        assert_eq!(connection.target, ConnectionTarget::Host { host: "localhost".into(), port: 27018 });
        assert_eq!(connection.authentication.kind, AuthenticationKind::BASIC);
        assert_eq!(connection.authentication.username.as_deref(), Some("root"));
        assert_eq!(connection.authentication.database.as_deref(), Some("admin"));
        assert_eq!(connection.options, ConnectionOptions::default());
        assert!(moved.is_empty());
    }

    #[test]
    fn turns_a_connection_string_into_a_uri_target_and_moves_its_password() {
        let (storage, moved) = migrate_v1(&v1(json!({
            "b": {
                "id": "b", "name": "Atlas", "uri": "mongodb+srv://ada:pw@cluster0.example.net/?retryWrites=true",
                "port": 27017, "connector": "MongoDb", "connectionType": "DnsSeedList",
                "authentication": { "kind": "NONE", "username": null, "password": null, "database": null }
            }
        })))
        .unwrap();

        let connection = &storage.connections["b"];
        assert_eq!(connection.target, ConnectionTarget::Uri { uri: "mongodb+srv://cluster0.example.net/?retryWrites=true".into() });
        assert_eq!(connection.authentication.kind, AuthenticationKind::BASIC);
        assert_eq!(connection.authentication.username.as_deref(), Some("ada"));
        assert_eq!(moved, vec![MovedPassword { connection_id: "b".into(), password: "pw".into() }]);
    }

    #[test]
    fn defaults_a_missing_port_and_authentication_and_takes_the_key_as_id() {
        let (storage, _) = migrate_v1(&v1(json!({ "k": { "name": "N", "uri": "db.local" } }))).unwrap();
        let connection = &storage.connections["k"];
        assert_eq!(connection.id, "k");
        assert_eq!(connection.target, ConnectionTarget::Host { host: "db.local".into(), port: 27017 });
        assert_eq!(connection.authentication, AuthenticationOptions::default());
    }

    #[test]
    fn drops_a_password_that_was_stored_in_the_file() {
        let (storage, _) = migrate_v1(&v1(json!({
            "a": { "name": "N", "uri": "h", "authentication": { "kind": "BASIC", "username": "u", "password": "leaked", "database": null } }
        })))
        .unwrap();
        assert_eq!(storage.connections["a"].authentication.password, None);
    }

    #[test]
    fn migrates_an_empty_file() {
        let (storage, moved) = migrate_v1(&v1(json!({}))).unwrap();
        assert!(storage.connections.is_empty());
        assert!(moved.is_empty());
    }

    #[test]
    fn refuses_connections_without_name_or_uri() {
        assert!(migrate_v1(&v1(json!({ "a": { "uri": "h" } }))).is_err());
        assert!(migrate_v1(&v1(json!({ "a": { "name": "N" } }))).is_err());
        assert!(migrate_v1(&json!({ "version": 1 })).is_err());
    }
}
