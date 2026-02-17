use std::error::Error;
use std::path::PathBuf;

use async_trait::async_trait;
use tokio::fs;

use crate::connector::models::authentication::AuthenticationKind;
use crate::connector::models::connection::Connection;
use crate::connector::models::create_connection::CreateConnection;
use crate::connector::repositories::connector_repository::ConnectorRepository;

pub struct FileRepository {
    base_path: PathBuf,
    credential_service: crate::connector::repositories::credentials::credential_service::CredentialService,
}

impl FileRepository {
    pub fn new(base_path: PathBuf, credential_service: crate::connector::repositories::credentials::credential_service::CredentialService) -> Self {
        FileRepository {
            base_path: base_path.join("connections.json"),
            credential_service,
        }
    }

    async fn write_all(
        &self,
        connections: &Vec<Connection>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        if let Some(parent) = self.base_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        let json_str = serde_json::to_string_pretty(connections)?;
        fs::write(&self.base_path, json_str).await?;
        Ok(())
    }
}

#[async_trait]
impl ConnectorRepository for FileRepository {
    async fn list(&self) -> Result<Vec<Connection>, Box<dyn Error + Send + Sync>> {
        let exists = fs::try_exists(&self.base_path).await?;

        if !exists {
            return Ok(Vec::new());
        }

        let content_str = fs::read_to_string(&self.base_path)
            .await
            .unwrap_or_else(|err| {
                log::error!("Could not read file content: {}", err);
                String::new()
            });

        if content_str.is_empty() {
            return Ok(Vec::new());
        }

        let mut connections: Vec<Connection> =
            serde_json::from_str(&content_str).unwrap_or_else(|err| {
                log::error!("Could not parse file content: {}", err);
                Vec::new()
            });

        for conn in &mut connections {
            if let AuthenticationKind::BASIC = conn.authentication.kind {
                if let Ok(password) = self.credential_service.get_password(&conn.id).await {
                    conn.authentication.password = Some(password);
                }
            }
        }

        Ok(connections)
    }

    async fn save(
        &self,
        mut connection: CreateConnection,
    ) -> Result<Connection, Box<dyn Error + Send + Sync>> {
        let mut list = self.list().await?;

        // we need to strip the password before writing it to the file
        let password = connection.authentication.password.take();

        let new_connection = Connection::from(connection);

        if let Some(pass) = password {
            self.credential_service
                .set_password(&new_connection.id, &pass)
                .await?;
        }

        list.push(new_connection.clone());

        self.write_all(&list).await?;

        return Ok(new_connection);
    }

    async fn get(&self, id: &str) -> Result<Connection, Box<dyn Error + Send + Sync>> {
        let mut connection = {
            let list = self.list().await?;
            list.into_iter()
                .find(|p| p.id == id)
                .ok_or_else(|| "Could not get Connection")?
        };

        if let AuthenticationKind::BASIC = connection.authentication.kind {
            if let Ok(password) = self.credential_service.get_password(id).await {
                connection.authentication.password = Some(password);
            }
        }

        Ok(connection)
    }

    async fn update(
        &self,
        id: &str,
        connection: &Connection,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {

        let mut list = self.list().await?;

        let item = list
            .iter_mut()
            .find(|p| p.id == id)
            .ok_or_else(|| format!("Connection with id '{}' not found", id))?;

        match connection.authentication.kind {
            AuthenticationKind::BASIC => {
                if let Some(password) = connection.authentication.password.clone() {
                    self.credential_service
                        .set_password(id, &password)
                        .await?;
                }
            }
            AuthenticationKind::NONE => {
                let _ = self.credential_service.delete_password(id).await;
            }
        }

        let mut clean_connection = connection.clone();
        clean_connection.authentication.password = None;

        *item = clean_connection;

        self.write_all(&list).await?;

        Ok(())
    }

    async fn delete(&self, id: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
        let mut list = self.list().await?;

        let position = list
            .iter()
            .position(|p| p.id == id)
            .ok_or_else(|| format!("Connection id {} not found", id))?;

        list.remove(position);
        self.credential_service.delete_password(id).await.ok();

        self.write_all(&list).await?;

        Ok(())
    }
}
