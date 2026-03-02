use crate::{
    connector::models::{authentication::AuthenticationKind, connection::Connection}, database::models::find_document::FindDocumentsResult, drivers::{
        driver::{DatabaseDriver, ProgressCallback, TestStage},
        errors::DriverError,
    }
};
use async_trait::async_trait;
use futures::StreamExt;
use mongodb::{bson::doc, Client};

pub struct MongoDbOfficialDriver {
    client: Option<Client>,
    connection: Connection,
}

impl MongoDbOfficialDriver {
    pub fn new(connection: Connection) -> Self {
        MongoDbOfficialDriver {
            client: None,
            connection,
        }
    }

    fn build_connection_string(&self) -> Result<String, DriverError> {
        let conn = &self.connection;
        let mut uri = match url::Url::parse(format!("mongodb://{}", &conn.uri).as_str()) {
            Ok(res) => res,
            Err(err) => {
                log::warn!("Uri malformed: {}", err);
                return Err(DriverError::UriMalformed(err.to_string()));
            }
        };

        uri.set_port(Some(conn.port)).map_err(|_| {
            DriverError::UriMalformed(format!("Could not set port to {}", conn.port))
        })?;

        let mut query: Vec<String> = Vec::new();

        match &conn.authentication.kind {
            AuthenticationKind::BASIC => {
                uri.set_username(conn.authentication.username.as_deref().unwrap_or("root"))
                    .map_err(|_| {
                        DriverError::UriMalformed(String::from("Could not set username"))
                    })?;
                uri.set_password(conn.authentication.password.as_deref())
                    .map_err(|_| {
                        DriverError::UriMalformed(String::from("Could not set password"))
                    })?;
            }
            AuthenticationKind::NONE => {}
        };

        let auth_db = conn.authentication.database.as_deref().unwrap_or("admin");

        if !auth_db.is_empty() {
            query.push(format!("authSource={}", auth_db));
        }

        uri.set_query(Some(query.join("&").as_str()));

        if cfg!(debug_assertions) {
            let mut log_uri = uri.clone();
            if log_uri.password().is_some() {
                let _ = log_uri.set_password(Some("***"));
            }
            log::debug!("Built connection query: {}", log_uri.as_str());
        }

        Ok(uri.to_string())
    }
}

#[async_trait]
impl DatabaseDriver for MongoDbOfficialDriver {
    async fn connect(&mut self) -> Result<(), DriverError> {
        if self.client.is_none() {
            let connection_string = self.build_connection_string()?;
            self.client = Some(
                Client::with_uri_str(&connection_string)
                    .await
                    .map_err(|e| DriverError::ConnectionFailed(e.to_string()))?,
            );
        }

        Ok(())
    }

    async fn disconnect(&mut self) -> Result<(), DriverError> {
        self.client = None;

        Ok(())
    }

    /// Testing Connection
    /// ## Steps
    /// 1. Initialize
    /// 2. Ping Database
    /// 3. Reading stats
    /// 4. Version detection
    /// 5. Connected
    async fn test_connection(
        &self,
        on_progress: ProgressCallback,
    ) -> Result<Vec<TestStage>, DriverError> {
        let mut stages: Vec<TestStage> = vec![
            TestStage::new(None, String::from("Initialize")),
            TestStage::new(None, String::from("Ping Database")),
            TestStage::new(None, String::from("Reading status")),
            TestStage::new(None, String::from("Detecting version")),
            TestStage::new(None, String::from("Connected")),
        ];

        // Stage 0: Initialize Connection
        let client_ref = self
            .client
            .as_ref()
            .ok_or(DriverError::ClientNotInitialized)?;
        stages[0].status = Some(true);
        on_progress(0, &stages[0]);

        let default_database_name = self
            .connection
            .authentication
            .database
            .as_ref()
            .map(|s| s.as_str())
            .unwrap_or("admin");

        let default_database = client_ref.database(default_database_name);

        // Stage 1: Ping Database
        default_database
            .run_command(doc! {"ping": 1})
            .await
            .map_err(|e| DriverError::PingFailed(e.to_string()))?;
        stages[1].status = Some(true);
        on_progress(1, &stages[1]);

        // Stage 2: Reading Server status
        let server_status = default_database
            .run_command(doc! {"serverStatus": 1})
            .await
            .map_err(|e| DriverError::ServerStatusFailed(e.to_string()))?;
        stages[2].status = Some(true);
        on_progress(2, &stages[2]);

        // Stage 3: Detecting MongoDB version
        if server_status.get_str("version").is_ok() {
            stages[3].status = Some(true);
        } else {
            stages[3].status = Some(false);
        }
        on_progress(3, &stages[3]);

        // Stage 4: Connected
        stages[4].status = Some(true);
        on_progress(4, &stages[4]);

        Ok(stages)
    }

    async fn list_databases(&self) -> Result<Vec<String>, DriverError> {
        let client = self
            .client
            .as_ref()
            .ok_or(DriverError::ClientNotInitialized)?;

        client
            .list_database_names()
            .await
            .map_err(|e| DriverError::ListDatabasesFailed(e.to_string()))
    }

    async fn list_collections(&self, database_name: &str) -> Result<Vec<String>, DriverError> {
        let client = self
            .client
            .as_ref()
            .ok_or(DriverError::ClientNotInitialized)?;

        let database = client.database(&database_name);

        database.list_collection_names().await.map_err(|e| {
            DriverError::ListCollectionsFailed(database_name.to_string(), e.to_string())
        })
    }

    async fn find_documents(
        &self,
        database_name: &str,
        collection_name: &str,
        filter: serde_json::Value,
        skip: u64,
        limit: i64
    ) -> Result<FindDocumentsResult, DriverError> {
        let client = self
            .client
            .as_ref()
            .ok_or(DriverError::ClientNotInitialized)?;

        let database = client.database(&database_name);
        let collection = database.collection::<mongodb::bson::Document>(&collection_name);

        let filter_doc = match mongodb::bson::to_document(&filter) {
            Ok(doc) => doc,
            Err(e) => {
                log::warn!("Failed to convert filter to BSON document: {}", e);
                return Err(DriverError::FindDocumentsFailed);
            }
        };

        let mut cursor = collection
            .find(filter_doc.clone())
            .skip(skip)
            .limit(limit)
            .allow_disk_use(true)
            .await
            .map_err(|_| DriverError::FindDocumentsFailed)?;

        let mut documents = Vec::new();
        while let Some(result) = cursor.next().await {
            match result {
                Ok(doc) => documents.push(serde_json::to_value(doc).unwrap_or_default()),
                Err(_) => {
                    log::warn!("Failed to read document from cursor");
                    return Err(DriverError::FindDocumentsFailed);
                }
            }
        }

        // Get total count of matching documents without skip/limit and give up ownership as we do not need it further
        let total_count = collection
            .count_documents(filter_doc)
            .await
            .map_err(|_| DriverError::FindDocumentsFailed)?;

        Ok(FindDocumentsResult { documents, total_count })
    }
}
