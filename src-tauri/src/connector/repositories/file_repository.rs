use std::any::Any;
use std::path::PathBuf;

use async_trait::async_trait;
use tokio::fs;

use crate::connector::commands::error::RepositoryError;
use crate::connector::models::authentication::AuthenticationKind;
use crate::connector::models::connection::Connection;
use crate::connector::models::create_connection::CreateConnection;
use crate::connector::repositories::connector_repository::ConnectorRepository;
use crate::connector::repositories::credentials::credential_service::CredentialService;
use crate::connector::repositories::storage_wrapper::{ConnectionStorage, STORAGE_VERSION};

pub struct FileRepository {
    base_path: PathBuf,
    pub credential_service: CredentialService,
}

impl FileRepository {
    pub fn new(base_path: PathBuf, credential_service: crate::connector::repositories::credentials::credential_service::CredentialService) -> Self {
        FileRepository {
            base_path: base_path.join("connections.json"),
            credential_service,
        }
    }
}

#[async_trait]
impl ConnectorRepository for FileRepository {
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    async fn list(&self) -> Result<Vec<Connection>, RepositoryError> {
        let storage = self.load_storage().await?;
        Ok(storage.connections.values().cloned().collect())
    }

    async fn save(
        &self,
        mut connection: CreateConnection,
    ) -> Result<Connection, RepositoryError> {
        let mut storage = self.load_storage().await?;

        let password = connection.authentication.password.take();
        let new_connection = Connection::from(connection);

        if let Some(pass) = password {
            self.credential_service
                .set_password(&new_connection.id, &pass)
                .await
                .map_err(|e| RepositoryError::Credential(e))?;
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

        match connection.authentication.kind {
            AuthenticationKind::BASIC => {
                if let Some(password) = &connection.authentication.password {
                    self.credential_service
                        .set_password(id, password)
                        .await
                        .map_err(|e| RepositoryError::Credential(e))?;
                }
            }
            AuthenticationKind::NONE => {
                let _ = self.credential_service.delete_password(id).await;
            }
        }

        let mut clean = connection.clone();
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

        let _ = self.credential_service.delete_password(id).await;

        self.write_storage(&storage).await?;

        Ok(())
    }

    async fn load_storage(&self) -> Result<ConnectionStorage, RepositoryError> {
        if !fs::try_exists(&self.base_path).await? {
            return Ok(ConnectionStorage::new());
        }

        let content = fs::read_to_string(&self.base_path).await?;

        if content.trim().is_empty() {
            return Ok(ConnectionStorage::new());
        }

        let storage: ConnectionStorage = serde_json::from_str(&content)?;

        if storage.version != STORAGE_VERSION {
            return Err(RepositoryError::MigrationError {
                from: storage.version,
                to: STORAGE_VERSION,
            });
        }

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
