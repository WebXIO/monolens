use crate::{
    connector::models::connection::Connection,
    database::models::find_document::{FindDocumentsResult, FindQuery},
    drivers::{
        client_options::client_options,
        documents::{document_to_json, json_to_document},
        driver::{DatabaseDriver, ProgressCallback, StageStatus, TestStage},
        errors::DriverError,
    }
};
use async_trait::async_trait;
use futures::StreamExt;
use mongodb::{
    bson::{doc, Document},
    Client,
};
use std::time::Duration;

const TEST_STAGES: [&str; 5] = [
    "Initialize",
    "Ping Database",
    "Reading status",
    "Detecting version",
    "Connected",
];

const MAX_LIMIT: i64 = 10_000;

/// Without a timeout of its own, a connection waits the driver's 30 s for a server.
/// A test should say sooner that nothing answers, e.g. on a wrong port.
const TEST_SERVER_SELECTION_TIMEOUT: Duration = Duration::from_secs(10);

/// The server's own message for a failed command, without the driver's wrapping.
fn find_failed(error: mongodb::error::Error) -> DriverError {
    let message = match error.kind.as_ref() {
        mongodb::error::ErrorKind::Command(command) => command.message.clone(),
        _ => error.to_string(),
    };
    log::warn!("find failed: {}", message);
    DriverError::FindDocumentsFailed(message)
}

/// MongoDB's error code 13, Unauthorized: the user lacks the privilege for a command.
fn is_unauthorized(error: &mongodb::error::Error) -> bool {
    matches!(error.kind.as_ref(), mongodb::error::ErrorKind::Command(command) if command.code == 13)
}

pub struct MongoDbOfficialDriver {
    client: Option<Client>,
    connection: Connection,
    fallback_timeout: Option<Duration>,
}

impl MongoDbOfficialDriver {
    pub fn new(connection: Connection) -> Self {
        MongoDbOfficialDriver {
            client: None,
            connection,
            fallback_timeout: None,
        }
    }

    /// A driver for the connection test, which gives up sooner when no server answers.
    pub fn for_test(connection: Connection) -> Self {
        MongoDbOfficialDriver {
            fallback_timeout: Some(TEST_SERVER_SELECTION_TIMEOUT),
            ..Self::new(connection)
        }
    }
}

#[async_trait]
impl DatabaseDriver for MongoDbOfficialDriver {
    async fn connect(&mut self) -> Result<(), DriverError> {
        if self.client.is_none() {
            let options = client_options(&self.connection, self.fallback_timeout).await?;
            // never the credential: hosts and settings only
            log::debug!(
                "Client for {}: hosts {:?}, tls {}, replica set {:?}, timeout {:?}, authentication {}",
                self.connection.name,
                options.hosts,
                options.tls.is_some(),
                options.repl_set_name,
                options.server_selection_timeout,
                options.credential.is_some(),
            );
            self.client = Some(
                Client::with_options(options)
                    .map_err(|e| DriverError::ConnectionFailed(e.to_string()))?,
            );
        }

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

        let database = client.database(database_name);

        database.list_collection_names().await.map_err(|e| {
            DriverError::ListCollectionsFailed(database_name.to_string(), e.to_string())
        })
    }

    async fn find_documents(&self, query: &FindQuery) -> Result<FindDocumentsResult, DriverError> {
        let client = self
            .client
            .as_ref()
            .ok_or(DriverError::ClientNotInitialized)?;

        if !(1..=MAX_LIMIT).contains(&query.limit) {
            return Err(DriverError::InvalidQuery(format!(
                "The limit must be between 1 and {MAX_LIMIT}."
            )));
        }

        let filter = json_to_document("filter", &query.filter)?;
        let sort = json_to_document("sort", &query.sort)?;
        let projection = json_to_document("projection", &query.projection)?;

        let collection = client
            .database(&query.database_name)
            .collection::<Document>(&query.collection_name);

        let find = async {
            let mut find = collection
                .find(filter.clone())
                .skip(query.skip)
                .limit(query.limit)
                .allow_disk_use(true);
            if !sort.is_empty() {
                find = find.sort(sort);
            }
            if !projection.is_empty() {
                find = find.projection(projection);
            }

            let mut cursor = find.await.map_err(find_failed)?;
            let mut documents = Vec::new();
            while let Some(document) = cursor.next().await {
                documents.push(document_to_json(document.map_err(find_failed)?));
            }
            Ok::<_, DriverError>(documents)
        };

        // an empty filter counts from the collection's metadata instead of scanning it
        let estimated = filter.is_empty();
        let count = async {
            if estimated {
                collection.estimated_document_count().await
            } else {
                collection.count_documents(filter.clone()).await
            }
            .map_err(find_failed)
        };

        let (documents, total_count) = futures::try_join!(find, count)?;

        Ok(FindDocumentsResult {
            documents,
            total_count,
            total_count_estimated: estimated,
        })
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

    /// Needs no MongoDB, only a port nothing listens on; slow, so it runs with the others.
    #[tokio::test]
    #[ignore]
    async fn the_test_gives_up_after_its_own_timeout_when_no_server_answers() {
        let port = std::net::TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port();
        let mut driver = MongoDbOfficialDriver::for_test(Connection {
            options: ConnectionOptions::default(),
            ..connection(ConnectionTarget::Host { host: "127.0.0.1".into(), port }, AuthenticationOptions::default())
        });
        driver.connect().await.unwrap();

        let started = std::time::Instant::now();
        let err = driver.test_connection(Box::new(|_, _| {})).await.unwrap_err();

        assert!(matches!(&err, DriverError::PingFailed(_)), "{err:?}");
        assert!(err.user_message().starts_with("Connection timed out"), "{}", err.user_message());
        let waited = started.elapsed();
        assert!(waited >= TEST_SERVER_SELECTION_TIMEOUT && waited < Duration::from_secs(20), "{waited:?}");
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

    fn users(filter: serde_json::Value, sort: serde_json::Value, projection: serde_json::Value, skip: u64, limit: i64) -> FindQuery {
        FindQuery {
            database_name: "shop".into(),
            collection_name: "users".into(),
            filter,
            sort,
            projection,
            skip,
            limit,
        }
    }

    /// Needs the collection shop.users with exactly { name: "Ada", age: 36 } and { name: "Bob", age: 19 }.
    #[tokio::test]
    #[ignore]
    async fn finds_with_sort_projection_and_paging_and_estimates_an_unfiltered_count() {
        use serde_json::json;
        let driver = connected(connection(host(), root(None))).await;

        let all = driver
            .find_documents(&users(json!({}), json!({ "age": 1 }), json!({ "name": 1, "_id": 0 }), 0, 10))
            .await
            .unwrap();
        assert_eq!(all.documents, vec![json!({ "name": "Bob" }), json!({ "name": "Ada" })]);
        assert_eq!(all.total_count, 2);
        assert!(all.total_count_estimated);

        let second = driver
            .find_documents(&users(serde_json::Value::Null, json!({ "age": 1 }), json!({ "name": 1, "_id": 0 }), 1, 1))
            .await
            .unwrap();
        assert_eq!(second.documents, vec![json!({ "name": "Ada" })]);
        assert_eq!(second.total_count, 2);
    }

    #[tokio::test]
    #[ignore]
    async fn counts_a_filter_exactly() {
        use serde_json::json;
        let driver = connected(connection(host(), root(None))).await;

        let older = driver
            .find_documents(&users(json!({ "age": { "$gt": 20 } }), json!(null), json!({ "_id": 0 }), 0, 10))
            .await
            .unwrap();

        assert_eq!(older.documents, vec![json!({ "name": "Ada", "age": 36 })]);
        assert_eq!(older.total_count, 1);
        assert!(!older.total_count_estimated);
    }

    #[tokio::test]
    #[ignore]
    async fn a_broken_query_returns_the_server_message() {
        use serde_json::json;
        let driver = connected(connection(host(), root(None))).await;

        let err = driver.find_documents(&users(json!({ "$foo": 1 }), json!(null), json!(null), 0, 10)).await.unwrap_err();

        assert!(err.user_message().contains("unknown top level operator: $foo"), "{}", err.user_message());
    }

    #[tokio::test]
    #[ignore]
    async fn a_limit_out_of_range_is_refused_before_the_server() {
        let driver = connected(connection(host(), root(None))).await;
        let err = driver.find_documents(&users(serde_json::Value::Null, serde_json::Value::Null, serde_json::Value::Null, 0, 0)).await.unwrap_err();
        assert!(matches!(err, DriverError::InvalidQuery(_)));
    }

    #[tokio::test]
    #[ignore]
    async fn reading_without_permission_says_so() {
        let reader = AuthenticationOptions {
            kind: AuthenticationKind::BASIC,
            username: Some("reader".into()),
            password: Some("readerpw".into()),
            database: Some("shop".into()),
        };
        let driver = connected(connection(host(), reader)).await;
        let mut query = users(serde_json::Value::Null, serde_json::Value::Null, serde_json::Value::Null, 0, 10);
        query.database_name = "other".into();

        let err = driver.find_documents(&query).await.unwrap_err();

        assert!(err.user_message().starts_with("Not allowed"), "{}", err.user_message());
    }

    #[tokio::test]
    #[ignore]
    async fn a_wrong_password_is_an_authentication_error() {
        let driver = connected(connection(host(), root(Some("wrong".into())))).await;
        let err = driver.list_databases().await.unwrap_err();
        assert!(err.user_message().starts_with("Authentication failed"), "{err:?}");
    }
}
