use std::error::Error;
use std::path::PathBuf;

use async_trait::async_trait;
use tokio::fs;

use crate::connector::models::connection::Connection;
use crate::connector::models::create_connection::CreateConnection;
use crate::connector::repositories::connector_repository::ConnectorRepository;

pub struct FileRepository {
    base_path: PathBuf,
}

impl FileRepository {
    pub fn new(base_path: PathBuf) -> Self {
        FileRepository {
            base_path: base_path.join("connections.json"),
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
                log::error!("Could not read file content {}", err);
                return String::new();
            });

        if content_str.is_empty() {
            return Ok(Vec::new());
        }

        let content: Vec<Connection> =
            serde_json::from_str(content_str.as_str()).unwrap_or_else(|err| {
                log::error!("Could not parse file content {}", err);
                return Vec::new();
            });

        return Ok(content);
    }

    async fn save(
        &self,
        connection: CreateConnection,
    ) -> Result<Connection, Box<dyn Error + Send + Sync>> {
        let mut list = self.list().await?;

        let new_connection = Connection::from(connection);

        list.push(new_connection);

        self.write_all(&list).await?;

        return Ok(list.pop().unwrap());
    }

    async fn get(&self, id: &str) -> Result<Connection, Box<dyn Error + Send + Sync>> {
        let list = self.list().await?;

        let item = list.into_iter().find(|p| p.id.eq(id));

        match item {
            Some(obj) => Ok(obj),
            _ => Err("Could not get Connection".into()),
        }
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

        *item = Connection {
            id: connection.id.to_string(),
            ..connection.clone()
        };

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

        self.write_all(&list).await?;

        Ok(())
    }
}
