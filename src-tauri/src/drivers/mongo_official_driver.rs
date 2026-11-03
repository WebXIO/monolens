use crate::{
    connector::models::connection::Connection, database::models::find_document::FindDocumentsResult, drivers::{
        client_options::client_options,
        driver::{DatabaseDriver, ProgressCallback, StageStatus, TestStage},
        errors::DriverError,
    }
};
use async_trait::async_trait;
use futures::StreamExt;
use mongodb::{bson::doc, Client};

const TEST_STAGES: [&str; 5] = [
    "Initialize",
    "Ping Database",
    "Reading status",
    "Detecting version",
    "Connected",
];

/// MongoDB's error code 13, Unauthorized: the user lacks the privilege for a command.
fn is_unauthorized(error: &mongodb::error::Error) -> bool {
    matches!(error.kind.as_ref(), mongodb::error::ErrorKind::Command(command) if command.code == 13)
}

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
    /// ## Stages
    /// 1. Initialize
    /// 2. Ping Database (also authenticates)
    /// 3. Reading status (`serverStatus`, skipped without the privilege)
    /// 4. Detecting version (`buildInfo`, needs no privilege)
    /// 5. Connected
    ///
    /// Every stage is reported as pending first, so the stage titles exist only here.
    async fn test_connection(
        &self,
        on_progress: ProgressCallback,
    ) -> Result<Vec<TestStage>, DriverError> {
        let mut stages: Vec<TestStage> = TEST_STAGES.iter().map(|title| TestStage::pending(title)).collect();
        for (index, stage) in stages.iter().enumerate() {
            on_progress(index, stage);
        }

        let mut report = |index: usize, status: StageStatus, detail: Option<String>| {
            stages[index].status = status;
            stages[index].detail = detail;
            on_progress(index, &stages[index]);
        };

        // Stage 0: Initialize Connection
        let Some(client_ref) = self.client.as_ref() else {
            report(0, StageStatus::Failed, None);
            return Err(DriverError::ClientNotInitialized);
        };
        report(0, StageStatus::Passed, None);

        let default_database_name = self
            .connection
            .authentication
            .database
            .as_deref()
            .filter(|name| !name.trim().is_empty())
            .unwrap_or("admin");

        let default_database = client_ref.database(default_database_name);

        // Stage 1: Ping Database
        if let Err(e) = default_database.run_command(doc! {"ping": 1}).await {
            report(1, StageStatus::Failed, None);
            return Err(DriverError::PingFailed(e.to_string()));
        }
        report(1, StageStatus::Passed, None);

        // Stage 2: Reading Server status
        match default_database.run_command(doc! {"serverStatus": 1}).await {
            Ok(_) => report(2, StageStatus::Passed, None),
            Err(e) if is_unauthorized(&e) => report(
                2,
                StageStatus::Skipped,
                Some(String::from("No permission to read the server status")),
            ),
            Err(e) => {
                report(2, StageStatus::Failed, None);
                return Err(DriverError::ServerStatusFailed(e.to_string()));
            }
        }

        // Stage 3: Detecting MongoDB version; an unknown version doesn't stop the test
        match default_database.run_command(doc! {"buildInfo": 1}).await {
            Ok(info) => match info.get_str("version") {
                Ok(version) => report(3, StageStatus::Passed, Some(format!("MongoDB {version}"))),
                Err(_) => report(3, StageStatus::Failed, Some(String::from("The server reported no version"))),
            },
            Err(e) => {
                log::warn!("buildInfo failed: {}", e);
                report(3, StageStatus::Failed, Some(String::from("Could not read the version")));
            }
        }

        // Stage 4: Connected
        report(4, StageStatus::Passed, None);

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

    fn statuses(stages: &[TestStage]) -> Vec<StageStatus> {
        stages.iter().map(|stage| stage.status).collect()
    }

    #[tokio::test]
    #[ignore]
    async fn the_connection_test_passes_every_stage_for_root_and_names_the_version() {
        let driver = connected(connection(host(), root(None))).await;
        let reported = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let sink = std::sync::Arc::clone(&reported);

        let stages = driver
            .test_connection(Box::new(move |index, stage| sink.lock().unwrap().push((index, stage.clone()))))
            .await
            .unwrap();

        assert_eq!(statuses(&stages), vec![StageStatus::Passed; 5]);
        assert!(stages[3].detail.as_deref().unwrap().starts_with("MongoDB "), "{:?}", stages[3]);
        let reported = reported.lock().unwrap();
        // five pending stages first, then one report per stage
        assert_eq!(reported.len(), 10);
        assert!(reported[..5].iter().all(|(_, stage)| stage.status == StageStatus::Pending));
    }

    /// Needs a user `reader` with password `readerpw` and the role read on the database `shop`.
    #[tokio::test]
    #[ignore]
    async fn a_user_without_the_server_status_privilege_skips_that_stage() {
        let reader = AuthenticationOptions {
            kind: AuthenticationKind::BASIC,
            username: Some("reader".into()),
            password: Some("readerpw".into()),
            database: Some("shop".into()),
        };
        let driver = connected(connection(host(), reader)).await;

        let stages = driver.test_connection(Box::new(|_, _| {})).await.unwrap();

        assert_eq!(
            statuses(&stages),
            vec![StageStatus::Passed, StageStatus::Passed, StageStatus::Skipped, StageStatus::Passed, StageStatus::Passed]
        );
    }

    #[tokio::test]
    #[ignore]
    async fn a_wrong_password_fails_the_ping_stage() {
        let driver = connected(connection(host(), root(Some("wrong".into())))).await;
        let failed = std::sync::Arc::new(std::sync::Mutex::new(None));
        let sink = std::sync::Arc::clone(&failed);

        let err = driver
            .test_connection(Box::new(move |index, stage| {
                if stage.status == StageStatus::Failed {
                    *sink.lock().unwrap() = Some(index);
                }
            }))
            .await
            .unwrap_err();

        assert!(err.user_message().starts_with("Authentication failed"), "{err:?}");
        assert_eq!(*failed.lock().unwrap(), Some(1));
    }

    #[tokio::test]
    #[ignore]
    async fn a_wrong_password_is_an_authentication_error() {
        let driver = connected(connection(host(), root(Some("wrong".into())))).await;
        let err = driver.list_databases().await.unwrap_err();
        assert!(err.user_message().starts_with("Authentication failed"), "{err:?}");
    }
}
