use crate::{
    connector::models::connection::Connection, database::models::find_document::FindDocumentsResult, drivers::{
        client_options::client_options,
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
}

#[async_trait]
impl DatabaseDriver for MongoDbOfficialDriver {
    async fn connect(&mut self) -> Result<(), DriverError> {
        if self.client.is_none() {
            let options = client_options(&self.connection).await?;
            // never the credential: hosts and settings only
            log::debug!(
                "Client for {}: hosts {:?}, tls {}, replica set {:?}, authentication {}",
                self.connection.name,
                options.hosts,
                options.tls.is_some(),
                options.repl_set_name,
                options.credential.is_some(),
            );
            self.client = Some(
                Client::with_options(options)
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

/// Against a real MongoDB, skipped by default. Run with a server and its root account:
/// MONGODB_TEST_HOST=127.0.0.1 MONGODB_TEST_PORT=27017 MONGODB_TEST_USER=root MONGODB_TEST_PASSWORD=… cargo test -- --ignored
#[cfg(test)]
mod integration {
    use super::*;
    use crate::connector::models::{
        authentication::{AuthenticationKind, AuthenticationOptions},
        connection::{ConnectionTarget, ConnectorKind},
        options::ConnectionOptions,
    };

    fn env(name: &str) -> String {
        std::env::var(name).unwrap_or_else(|_| panic!("{name} is not set"))
    }

    fn root(password: Option<String>) -> AuthenticationOptions {
        AuthenticationOptions {
            kind: AuthenticationKind::BASIC,
            username: Some(env("MONGODB_TEST_USER")),
            password: Some(password.unwrap_or_else(|| env("MONGODB_TEST_PASSWORD"))),
            database: Some("admin".into()),
        }
    }

    fn host() -> ConnectionTarget {
        ConnectionTarget::Host {
            host: env("MONGODB_TEST_HOST"),
            port: env("MONGODB_TEST_PORT").parse().unwrap(),
        }
    }

    fn connection(target: ConnectionTarget, authentication: AuthenticationOptions) -> Connection {
        Connection {
            id: "integration".into(),
            name: "integration".into(),
            connector: ConnectorKind::MongoDb,
            target,
            authentication,
            options: ConnectionOptions {
                server_selection_timeout_secs: Some(5),
                ..ConnectionOptions::default()
            },
        }
    }

    async fn connected(connection: Connection) -> MongoDbOfficialDriver {
        let mut driver = MongoDbOfficialDriver::new(connection);
        driver.connect().await.unwrap();
        driver
    }

    #[tokio::test]
    #[ignore]
    async fn connects_by_host_and_lists_databases() {
        let driver = connected(connection(host(), root(None))).await;
        let databases = driver.list_databases().await.unwrap();
        assert!(databases.contains(&"admin".to_string()), "{databases:?}");
    }

    #[tokio::test]
    #[ignore]
    async fn connects_by_connection_string_with_separate_credentials() {
        let uri = format!(
            "mongodb://{}:{}/?serverSelectionTimeoutMS=5000",
            env("MONGODB_TEST_HOST"),
            env("MONGODB_TEST_PORT")
        );
        let driver = connected(connection(ConnectionTarget::Uri { uri }, root(None))).await;
        assert!(driver.list_databases().await.unwrap().contains(&"admin".to_string()));
    }

    #[tokio::test]
    #[ignore]
    async fn a_wrong_password_is_an_authentication_error() {
        let driver = connected(connection(host(), root(Some("wrong".into())))).await;
        let err = driver.list_databases().await.unwrap_err();
        assert!(err.user_message().starts_with("Authentication failed"), "{err:?}");
    }
}
