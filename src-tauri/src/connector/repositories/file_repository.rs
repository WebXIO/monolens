use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::fs;

use crate::connector::commands::error::{CredentialError, RepositoryError};
use crate::connector::models::authentication::AuthenticationKind;
use crate::connector::models::connection::Connection;
use crate::connector::models::create_connection::CreateConnection;
use crate::connector::models::normalize::normalize;
use crate::connector::repositories::connector_repository::ConnectorRepository;
use crate::connector::repositories::credentials::credential_store::CredentialStore;
use crate::connector::repositories::migrations::migrate_v1;
use crate::connector::repositories::storage_wrapper::{ConnectionStorage, STORAGE_VERSION};

pub struct FileRepository {
    base_path: PathBuf,
    credentials: Arc<dyn CredentialStore>,
}

impl FileRepository {
    pub fn new(base_path: PathBuf, credentials: Arc<dyn CredentialStore>) -> Self {
        FileRepository {
            base_path: base_path.join("connections.json"),
            credentials,
        }
    }

    async fn load_storage(&self) -> Result<ConnectionStorage, RepositoryError> {
        if !fs::try_exists(&self.base_path).await? {
            return Ok(ConnectionStorage::new());
        }

        let content = fs::read_to_string(&self.base_path).await?;

        if content.trim().is_empty() {
            return Ok(ConnectionStorage::new());
        }

        let value: serde_json::Value = serde_json::from_str(&content)?;
        let version = value.get("version").and_then(serde_json::Value::as_u64);

        match version {
            Some(v) if v == u64::from(STORAGE_VERSION) => Ok(serde_json::from_value(value)?),
            Some(1) => self.migrate_from_v1(&content, &value).await,
            other => Err(RepositoryError::MigrationError {
                from: other.and_then(|v| u32::try_from(v).ok()).unwrap_or(0),
                to: STORAGE_VERSION,
            }),
        }
    }

    async fn migrate_from_v1(
        &self,
        content: &str,
        value: &serde_json::Value,
    ) -> Result<ConnectionStorage, RepositoryError> {
        let (storage, moved) = migrate_v1(value)?;

        for password in moved {
            self.credentials
                .set_password(&password.connection_id, &password.password)
                .await?;
        }

        // the old file stays next to the new one, in case the migration loses something
        fs::write(self.base_path.with_file_name("connections.v1.json"), content).await?;
        self.write_storage(&storage).await?;
        log::info!("Migrated connections.json from version 1 to {}", STORAGE_VERSION);

        Ok(storage)
    }

    async fn write_storage(&self, storage: &ConnectionStorage) -> Result<(), RepositoryError> {
        if let Some(parent) = self.base_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        let json = serde_json::to_string_pretty(storage)?;
        fs::write(&self.base_path, json).await?;

        Ok(())
    }
}

#[async_trait]
impl ConnectorRepository for FileRepository {
    async fn list(&self) -> Result<Vec<Connection>, RepositoryError> {
        let storage = self.load_storage().await?;
        Ok(storage.connections.values().cloned().collect())
    }

    async fn save(
        &self,
        mut connection: CreateConnection,
    ) -> Result<Connection, RepositoryError> {
        let mut storage = self.load_storage().await?;

        let (target, authentication) =
            normalize(&connection.name, connection.target, connection.authentication)
                .map_err(RepositoryError::Invalid)?;
        connection.target = target;
        connection.authentication = authentication;

        let password = connection.authentication.password.take();
        let new_connection = Connection::from(connection);

        if let Some(pass) = password {
            self.credentials
                .set_password(&new_connection.id, &pass)
                .await?;
        }

        storage
            .connections
            .insert(new_connection.id.clone(), new_connection.clone());

        self.write_storage(&storage).await?;

        Ok(new_connection)
    }

    async fn get(&self, id: &str) -> Result<Connection, RepositoryError> {
        let storage = self.load_storage().await?;

        storage
            .connections
            .get(id)
            .cloned()
            .ok_or_else(|| RepositoryError::NotFound(id.to_string()))
    }

    async fn update(
        &self,
        id: &str,
        connection: &Connection,
    ) -> Result<(), RepositoryError> {

        let mut storage = self.load_storage().await?;

        let existing = storage
            .connections
            .get_mut(id)
            .ok_or_else(|| RepositoryError::NotFound(id.to_string()))?;

        let (target, authentication) = normalize(
            &connection.name,
            connection.target.clone(),
            connection.authentication.clone(),
        )
        .map_err(RepositoryError::Invalid)?;
        let mut connection = connection.clone();
        connection.target = target;
        connection.authentication = authentication;

        match connection.authentication.kind {
            AuthenticationKind::BASIC => {
                // without a new password the stored one stays
                if let Some(password) = &connection.authentication.password {
                    self.credentials.set_password(id, password).await?;
                }
            }
            AuthenticationKind::NONE => {
                let _ = self.credentials.delete_password(id).await;
            }
        }

        let mut clean = connection;
        clean.id = id.to_string();
        clean.authentication.password = None;

        *existing = clean;

        self.write_storage(&storage).await?;

        Ok(())
    }

    async fn delete(&self, id: &str) -> Result<(), RepositoryError> {
        let mut storage = self.load_storage().await?;

        if storage.connections.remove(id).is_none() {
            return Err(RepositoryError::NotFound(id.to_string()));
        }

        let _ = self.credentials.delete_password(id).await;

        self.write_storage(&storage).await?;

        Ok(())
    }

    async fn get_password(&self, id: &str) -> Result<Option<String>, RepositoryError> {
        match self.credentials.get_password(id).await {
            Ok(password) => Ok(Some(password)),
            Err(CredentialError::NotFound(_)) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    async fn hydrate(&self, connection: &Connection) -> Result<Connection, RepositoryError> {
        let mut hydrated = connection.clone();
        if let AuthenticationKind::BASIC = hydrated.authentication.kind {
            hydrated.authentication.password =
                Some(self.credentials.get_password(&hydrated.id).await?);
        }
        Ok(hydrated)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connector::models::authentication::AuthenticationOptions;
    use crate::connector::models::connection::{ConnectionTarget, ConnectorKind};
    use crate::connector::models::options::ConnectionOptions;
    use crate::connector::repositories::credentials::credential_store::memory::MemoryCredentialStore;

    /// A directory of its own per test, removed again when the test ends.
    struct TempDir(PathBuf);

    impl TempDir {
        fn new() -> Self {
            let dir = std::env::temp_dir().join(format!("monolens-test-{}", uuid::Uuid::new_v4()));
            std::fs::create_dir_all(&dir).unwrap();
            TempDir(dir)
        }

        fn file(&self) -> PathBuf {
            self.0.join("connections.json")
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn setup() -> (TempDir, Arc<MemoryCredentialStore>, FileRepository) {
        let dir = TempDir::new();
        let store = Arc::new(MemoryCredentialStore::default());
        let repository = FileRepository::new(dir.0.clone(), store.clone());
        (dir, store, repository)
    }

    fn create(password: Option<&str>) -> CreateConnection {
        CreateConnection {
            name: "Local".into(),
            connector: ConnectorKind::MongoDb,
            target: ConnectionTarget::Host { host: "localhost".into(), port: 27017 },
            options: ConnectionOptions::default(),
            authentication: AuthenticationOptions {
                kind: if password.is_some() {
                    AuthenticationKind::BASIC
                } else {
                    AuthenticationKind::NONE
                },
                username: password.map(|_| "root".to_string()),
                password: password.map(String::from),
                database: Some("admin".into()),
            },
        }
    }

    #[tokio::test]
    async fn lists_nothing_without_a_file() {
        let (_dir, _store, repository) = setup();
        assert!(repository.list().await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn lists_nothing_for_an_empty_file() {
        let (dir, _store, repository) = setup();
        std::fs::write(dir.file(), "  \n").unwrap();
        assert!(repository.list().await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn save_keeps_the_password_out_of_the_file() {
        let (dir, store, repository) = setup();

        let saved = repository.save(create(Some("s3cr3t"))).await.unwrap();

        assert!(!saved.id.is_empty());
        assert_eq!(saved.authentication.password, None);
        assert_eq!(store.stored(&saved.id).as_deref(), Some("s3cr3t"));
        let file = std::fs::read_to_string(dir.file()).unwrap();
        assert!(!file.contains("s3cr3t"));
        assert!(file.contains("\"version\": 2"));
        assert_eq!(repository.get(&saved.id).await.unwrap().name, "Local");
    }

    #[tokio::test]
    async fn save_without_authentication_stores_no_password() {
        let (_dir, store, repository) = setup();
        let saved = repository.save(create(None)).await.unwrap();
        assert_eq!(store.stored(&saved.id), None);
    }

    #[tokio::test]
    async fn save_fails_when_the_keyring_fails_and_writes_nothing() {
        let (dir, store, repository) = setup();
        store.fail_from_now_on();

        let result = repository.save(create(Some("pw"))).await;

        assert!(matches!(result, Err(RepositoryError::Credential(_))));
        assert!(!dir.file().exists());
    }

    #[tokio::test]
    async fn get_of_an_unknown_id_is_not_found() {
        let (_dir, _store, repository) = setup();
        assert!(matches!(
            repository.get("nope").await,
            Err(RepositoryError::NotFound(id)) if id == "nope"
        ));
    }

    #[tokio::test]
    async fn update_with_a_password_replaces_it() {
        let (dir, store, repository) = setup();
        let saved = repository.save(create(Some("old"))).await.unwrap();

        let mut changed = saved.clone();
        changed.name = "Renamed".into();
        changed.authentication.password = Some("new".into());
        repository.update(&saved.id, &changed).await.unwrap();

        assert_eq!(store.stored(&saved.id).as_deref(), Some("new"));
        assert_eq!(repository.get(&saved.id).await.unwrap().name, "Renamed");
        assert!(!std::fs::read_to_string(dir.file()).unwrap().contains("new\""));
    }

    #[tokio::test]
    async fn update_without_a_password_keeps_the_stored_one() {
        let (_dir, store, repository) = setup();
        let saved = repository.save(create(Some("old"))).await.unwrap();

        repository.update(&saved.id, &saved).await.unwrap();

        assert_eq!(store.stored(&saved.id).as_deref(), Some("old"));
    }

    #[tokio::test]
    async fn update_to_no_authentication_deletes_the_password() {
        let (_dir, store, repository) = setup();
        let saved = repository.save(create(Some("old"))).await.unwrap();

        let mut changed = saved.clone();
        changed.authentication.kind = AuthenticationKind::NONE;
        repository.update(&saved.id, &changed).await.unwrap();

        assert_eq!(store.stored(&saved.id), None);
    }

    #[tokio::test]
    async fn update_keeps_the_id_of_the_path() {
        let (_dir, _store, repository) = setup();
        let saved = repository.save(create(None)).await.unwrap();

        let mut changed = saved.clone();
        changed.id = "something-else".into();
        repository.update(&saved.id, &changed).await.unwrap();

        assert_eq!(repository.get(&saved.id).await.unwrap().id, saved.id);
    }

    #[tokio::test]
    async fn update_of_an_unknown_id_is_not_found() {
        let (_dir, _store, repository) = setup();
        let saved = repository.save(create(None)).await.unwrap();
        assert!(matches!(
            repository.update("nope", &saved).await,
            Err(RepositoryError::NotFound(_))
        ));
    }

    #[tokio::test]
    async fn delete_removes_connection_and_password() {
        let (_dir, store, repository) = setup();
        let saved = repository.save(create(Some("pw"))).await.unwrap();

        repository.delete(&saved.id).await.unwrap();

        assert!(repository.list().await.unwrap().is_empty());
        assert_eq!(store.stored(&saved.id), None);
    }

    #[tokio::test]
    async fn delete_works_without_a_stored_password() {
        let (_dir, _store, repository) = setup();
        let saved = repository.save(create(None)).await.unwrap();
        repository.delete(&saved.id).await.unwrap();
        assert!(repository.list().await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn delete_of_an_unknown_id_is_not_found() {
        let (_dir, _store, repository) = setup();
        assert!(matches!(
            repository.delete("nope").await,
            Err(RepositoryError::NotFound(_))
        ));
    }

    #[tokio::test]
    async fn get_password_is_none_when_nothing_is_stored() {
        let (_dir, store, repository) = setup();
        let saved = repository.save(create(None)).await.unwrap();
        assert_eq!(repository.get_password(&saved.id).await.unwrap(), None);

        store.insert(&saved.id, "pw");
        assert_eq!(repository.get_password(&saved.id).await.unwrap().as_deref(), Some("pw"));
    }

    #[tokio::test]
    async fn get_password_reports_a_failing_keyring() {
        let (_dir, store, repository) = setup();
        store.fail_from_now_on();
        assert!(matches!(
            repository.get_password("x").await,
            Err(RepositoryError::Credential(CredentialError::Keyring(_)))
        ));
    }

    #[tokio::test]
    async fn hydrate_fills_the_password_for_basic_authentication() {
        let (_dir, _store, repository) = setup();
        let saved = repository.save(create(Some("pw"))).await.unwrap();

        let hydrated = repository.hydrate(&saved).await.unwrap();

        assert_eq!(hydrated.authentication.password.as_deref(), Some("pw"));
    }

    #[tokio::test]
    async fn hydrate_leaves_connections_without_authentication_alone() {
        let (_dir, store, repository) = setup();
        let saved = repository.save(create(None)).await.unwrap();
        store.fail_from_now_on();

        let hydrated = repository.hydrate(&saved).await.unwrap();

        assert_eq!(hydrated.authentication.password, None);
    }

    #[tokio::test]
    async fn hydrate_fails_when_the_password_is_missing() {
        let (_dir, store, repository) = setup();
        let saved = repository.save(create(Some("pw"))).await.unwrap();
        store.fail_from_now_on();
        assert!(repository.hydrate(&saved).await.is_err());
    }

    #[tokio::test]
    async fn a_file_of_another_version_is_refused() {
        let (dir, _store, repository) = setup();
        std::fs::write(dir.file(), r#"{ "version": 99, "connections": {} }"#).unwrap();
        assert!(matches!(
            repository.list().await,
            Err(RepositoryError::MigrationError { from: 99, to: STORAGE_VERSION })
        ));
    }

    #[tokio::test]
    async fn a_corrupt_file_is_a_serialization_error() {
        let (dir, _store, repository) = setup();
        std::fs::write(dir.file(), "{ not json").unwrap();
        assert!(matches!(
            repository.list().await,
            Err(RepositoryError::Serialization(_))
        ));
    }

    #[tokio::test]
    async fn save_moves_credentials_out_of_a_connection_string() {
        let (dir, store, repository) = setup();
        let mut connection = create(None);
        connection.target = ConnectionTarget::Uri { uri: "mongodb://ada:s3cr3t@h1,h2/?replicaSet=rs".into() };

        let saved = repository.save(connection).await.unwrap();

        assert_eq!(saved.target, ConnectionTarget::Uri { uri: "mongodb://h1,h2/?replicaSet=rs".into() });
        assert_eq!(saved.authentication.kind, AuthenticationKind::BASIC);
        assert_eq!(saved.authentication.username.as_deref(), Some("ada"));
        assert_eq!(store.stored(&saved.id).as_deref(), Some("s3cr3t"));
        assert!(!std::fs::read_to_string(dir.file()).unwrap().contains("s3cr3t"));
    }

    #[tokio::test]
    async fn save_refuses_an_invalid_connection_with_its_reason() {
        let (dir, _store, repository) = setup();
        let mut connection = create(None);
        connection.target = ConnectionTarget::Host { host: "mongodb://h".into(), port: 27017 };

        let result = repository.save(connection).await;

        assert!(matches!(result, Err(RepositoryError::Invalid(reason)) if reason.contains("connection string")));
        assert!(!dir.file().exists());
    }

    #[tokio::test]
    async fn update_refuses_an_invalid_connection() {
        let (_dir, _store, repository) = setup();
        let saved = repository.save(create(None)).await.unwrap();
        let mut changed = saved.clone();
        changed.name = "  ".into();
        assert!(matches!(repository.update(&saved.id, &changed).await, Err(RepositoryError::Invalid(_))));
    }

    #[tokio::test]
    async fn a_version_1_file_is_migrated_written_back_and_kept_as_backup() {
        let (dir, store, repository) = setup();
        let v1 = r#"{ "version": 1, "connections": {
            "a": { "id": "a", "name": "Local", "uri": "localhost", "port": 27017, "connector": "MongoDb",
                   "connectionType": "Standalone",
                   "authentication": { "kind": "BASIC", "username": "root", "password": null, "database": "admin" } },
            "b": { "id": "b", "name": "Atlas", "uri": "mongodb+srv://ada:pw@c.example.net", "port": 27017,
                   "connector": "MongoDb", "connectionType": "DnsSeedList",
                   "authentication": { "kind": "NONE", "username": null, "password": null, "database": null } } } }"#;
        std::fs::write(dir.file(), v1).unwrap();
        store.insert("a", "kept");

        let mut connections = repository.list().await.unwrap();
        connections.sort_by(|x, y| x.id.cmp(&y.id));

        assert_eq!(connections[0].target, ConnectionTarget::Host { host: "localhost".into(), port: 27017 });
        assert_eq!(connections[1].target, ConnectionTarget::Uri { uri: "mongodb+srv://c.example.net".into() });
        assert_eq!(store.stored("a").as_deref(), Some("kept"));
        assert_eq!(store.stored("b").as_deref(), Some("pw"));
        let file = std::fs::read_to_string(dir.file()).unwrap();
        assert!(file.contains("\"version\": 2"));
        assert!(!file.contains("connectionType"));
        assert_eq!(std::fs::read_to_string(dir.0.join("connections.v1.json")).unwrap(), v1);

        // the second load reads version 2 and doesn't migrate again
        assert_eq!(repository.list().await.unwrap().len(), 2);
    }

    #[tokio::test]
    async fn a_file_without_version_is_refused() {
        let (dir, _store, repository) = setup();
        std::fs::write(dir.file(), r#"{ "connections": {} }"#).unwrap();
        assert!(matches!(
            repository.list().await,
            Err(RepositoryError::MigrationError { from: 0, .. })
        ));
    }
}
