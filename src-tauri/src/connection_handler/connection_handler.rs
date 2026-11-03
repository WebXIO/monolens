use crate::{
    connector::{
        commands::error::RepositoryError, models::connection::Connection,
        repositories::connector_repository::ConnectorRepository,
    },
    drivers::{driver::DatabaseDriver, errors::DriverError, factory::DatabaseDriverFactory},
};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

/// The mongodb client is safe to share, so several commands of one connection run at the same time.
pub type SharedDriver = Arc<dyn DatabaseDriver>;
type DriverFactory = Box<dyn Fn(Connection) -> Result<Box<dyn DatabaseDriver>, DriverError> + Send + Sync>;

/// One connected driver per saved connection, created on first use.
/// A driver leaves the pool on disconnect, when its connection is edited or deleted,
/// and when its first command fails, so the next use builds it from the saved settings again.
pub struct ConnectionHandler {
    connection_pool: Mutex<HashMap<String, SharedDriver>>,
    factory: DriverFactory,
}

impl Default for ConnectionHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl ConnectionHandler {
    pub fn new() -> Self {
        Self::with_factory(Box::new(DatabaseDriverFactory::create))
    }

    pub fn with_factory(factory: DriverFactory) -> Self {
        ConnectionHandler {
            connection_pool: Mutex::new(HashMap::new()),
            factory,
        }
    }

    /// The saved connection with its password, for a driver.
    async fn load(
        repository: &dyn ConnectorRepository,
        connection_id: &str,
    ) -> Result<Connection, DriverError> {
        let connection = repository.get(connection_id).await.map_err(|e| match e {
            RepositoryError::NotFound(id) => DriverError::ConnectionNotFound(id),
            other => DriverError::StorageFailed(other.to_string()),
        })?;

        repository
            .hydrate(&connection)
            .await
            .map_err(|e| DriverError::CredentialsUnavailable(e.to_string()))
    }

    pub async fn get_or_connect(
        &self,
        repository: &dyn ConnectorRepository,
        connection_id: &str,
    ) -> Result<SharedDriver, DriverError> {
        if let Some(driver) = self.connection_pool.lock().await.get(connection_id) {
            return Ok(Arc::clone(driver));
        }

        // loading (keyring) and connecting (dns for srv strings) can take long;
        // the pool stays unlocked meanwhile, so other connections don't wait
        let connection = Self::load(repository, connection_id).await?;
        let mut driver = (self.factory)(connection)?;
        driver.connect().await?;
        let shared_driver: SharedDriver = Arc::from(driver);

        // a parallel call may have connected the same connection first; keep that driver
        let mut pool = self.connection_pool.lock().await;
        Ok(Arc::clone(
            pool.entry(connection_id.to_string()).or_insert(shared_driver),
        ))
    }

    /// Connects and lists the databases. The client only reaches the server with this first
    /// command, so a failure (wrong password, host down) drops the driver again.
    pub async fn open(
        &self,
        repository: &dyn ConnectorRepository,
        connection_id: &str,
    ) -> Result<Vec<String>, DriverError> {
        let driver = self.get_or_connect(repository, connection_id).await?;
        let result = driver.list_databases().await;

        if result.is_err() {
            self.disconnect(connection_id).await?;
        }

        result
    }

    /// Drops the driver; its client closes once running commands finish with it.
    pub async fn disconnect(&self, connection_id: &str) -> Result<(), DriverError> {
        self.connection_pool.lock().await.remove(connection_id);
        Ok(())
    }

    #[cfg(test)]
    async fn is_pooled(&self, connection_id: &str) -> bool {
        self.connection_pool.lock().await.contains_key(connection_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connector::commands::error::CredentialError;
    use crate::connector::models::{
        authentication::{AuthenticationKind, AuthenticationOptions},
        connection::{ConnectionTarget, ConnectorKind},
        create_connection::CreateConnection,
        options::ConnectionOptions,
    };
    use crate::database::models::find_document::{FindDocumentsResult, FindQuery};
    use crate::drivers::driver::{ProgressCallback, TestStage};
    use async_trait::async_trait;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[derive(Default)]
    struct Calls {
        created: AtomicUsize,
    }

    struct FakeDriver {
        connection: Connection,
    }

    #[async_trait]
    impl DatabaseDriver for FakeDriver {
        async fn connect(&mut self) -> Result<(), DriverError> {
            if self.connection.name == "refuses" {
                return Err(DriverError::UriMalformed("bad".into()));
            }
            Ok(())
        }

        async fn test_connection(&self, _: ProgressCallback) -> Result<Vec<TestStage>, DriverError> {
            Ok(vec![])
        }

        async fn list_databases(&self) -> Result<Vec<String>, DriverError> {
            match self.connection.authentication.password.as_deref() {
                Some("wrong") => Err(DriverError::ListDatabasesFailed("Authentication failed".into())),
                _ => Ok(vec![format!("db of {}", self.connection.name)]),
            }
        }

        async fn list_collections(&self, _: &str) -> Result<Vec<String>, DriverError> {
            Ok(vec![])
        }

        async fn find_documents(&self, _: &FindQuery) -> Result<FindDocumentsResult, DriverError> {
            Ok(FindDocumentsResult { documents: vec![], total_count: 0, total_count_estimated: false })
        }
    }

    /// Connections and passwords in memory, standing in for the file repository.
    #[derive(Default)]
    struct MemoryRepository {
        connections: std::sync::Mutex<HashMap<String, Connection>>,
        passwords: std::sync::Mutex<HashMap<String, String>>,
    }

    impl MemoryRepository {
        fn add(&self, id: &str, name: &str, password: Option<&str>) {
            let connection = Connection {
                id: id.into(),
                name: name.into(),
                connector: ConnectorKind::MongoDb,
                target: ConnectionTarget::Host { host: "h".into(), port: 1 },
                authentication: AuthenticationOptions {
                    kind: if password.is_some() { AuthenticationKind::BASIC } else { AuthenticationKind::NONE },
                    username: password.map(|_| "u".to_string()),
                    password: None,
                    database: None,
                },
                options: ConnectionOptions::default(),
            };
            self.connections.lock().unwrap().insert(id.into(), connection);
            if let Some(password) = password {
                self.passwords.lock().unwrap().insert(id.into(), password.into());
            }
        }

        fn set_password(&self, id: &str, password: &str) {
            self.passwords.lock().unwrap().insert(id.into(), password.into());
        }

        fn forget_password(&self, id: &str) {
            self.passwords.lock().unwrap().remove(id);
        }
    }

    #[async_trait]
    impl ConnectorRepository for MemoryRepository {
        async fn list(&self) -> Result<Vec<Connection>, RepositoryError> {
            Ok(self.connections.lock().unwrap().values().cloned().collect())
        }
        async fn get(&self, id: &str) -> Result<Connection, RepositoryError> {
            self.connections.lock().unwrap().get(id).cloned().ok_or_else(|| RepositoryError::NotFound(id.into()))
        }
        async fn save(&self, _: CreateConnection) -> Result<Connection, RepositoryError> {
            unimplemented!()
        }
        async fn update(&self, _: &str, _: &Connection) -> Result<(), RepositoryError> {
            unimplemented!()
        }
        async fn delete(&self, _: &str) -> Result<(), RepositoryError> {
            unimplemented!()
        }
        async fn get_password(&self, id: &str) -> Result<Option<String>, RepositoryError> {
            Ok(self.passwords.lock().unwrap().get(id).cloned())
        }
        async fn hydrate(&self, connection: &Connection) -> Result<Connection, RepositoryError> {
            let mut hydrated = connection.clone();
            if hydrated.authentication.kind == AuthenticationKind::BASIC {
                let password = self.passwords.lock().unwrap().get(&connection.id).cloned()
                    .ok_or_else(|| RepositoryError::Credential(CredentialError::NotFound(connection.id.clone())))?;
                hydrated.authentication.password = Some(password);
            }
            Ok(hydrated)
        }
    }

    fn handler() -> (ConnectionHandler, Arc<Calls>) {
        let calls = Arc::new(Calls::default());
        let factory_calls = Arc::clone(&calls);
        let handler = ConnectionHandler::with_factory(Box::new(move |connection| {
            factory_calls.created.fetch_add(1, Ordering::SeqCst);
            Ok(Box::new(FakeDriver { connection }) as Box<dyn DatabaseDriver>)
        }));
        (handler, calls)
    }

    #[tokio::test]
    async fn creates_a_driver_once_and_reuses_it() {
        let (handler, calls) = handler();
        let repository = MemoryRepository::default();
        repository.add("a", "A", None);

        assert_eq!(handler.open(&repository, "a").await.unwrap(), vec!["db of A"]);
        handler.get_or_connect(&repository, "a").await.unwrap();

        assert_eq!(calls.created.load(Ordering::SeqCst), 1);
        assert!(handler.is_pooled("a").await);
    }

    #[tokio::test]
    async fn disconnect_drops_the_driver_and_the_next_use_reads_the_settings_again() {
        let (handler, calls) = handler();
        let repository = MemoryRepository::default();
        repository.add("a", "A", None);
        handler.open(&repository, "a").await.unwrap();

        repository.add("a", "Renamed", None);
        handler.disconnect("a").await.unwrap();

        assert!(!handler.is_pooled("a").await);
        assert_eq!(handler.open(&repository, "a").await.unwrap(), vec!["db of Renamed"]);
        assert_eq!(calls.created.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn disconnecting_an_unknown_connection_does_nothing() {
        let (handler, _calls) = handler();
        handler.disconnect("nope").await.unwrap();
        assert!(!handler.is_pooled("nope").await);
    }

    #[tokio::test]
    async fn two_commands_of_one_connection_share_one_driver_at_the_same_time() {
        let (handler, calls) = handler();
        let repository = MemoryRepository::default();
        repository.add("a", "A", None);

        let (first, second) = tokio::join!(
            handler.get_or_connect(&repository, "a"),
            handler.get_or_connect(&repository, "a"),
        );

        // both may build a driver, but the pool keeps one and both callers get it
        assert!(Arc::ptr_eq(&first.unwrap(), &second.unwrap()));
        assert!(calls.created.load(Ordering::SeqCst) <= 2);
        assert_eq!(handler.connection_pool.lock().await.len(), 1);
    }

    #[tokio::test]
    async fn a_failing_first_command_drops_the_driver_so_a_new_password_works() {
        let (handler, calls) = handler();
        let repository = MemoryRepository::default();
        repository.add("a", "A", Some("wrong"));

        assert!(matches!(handler.open(&repository, "a").await, Err(DriverError::ListDatabasesFailed(_))));
        assert!(!handler.is_pooled("a").await);

        repository.set_password("a", "right");
        assert_eq!(handler.open(&repository, "a").await.unwrap(), vec!["db of A"]);
        assert_eq!(calls.created.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn an_unknown_connection_is_not_found() {
        let (handler, calls) = handler();
        let repository = MemoryRepository::default();
        assert!(matches!(handler.open(&repository, "x").await, Err(DriverError::ConnectionNotFound(id)) if id == "x"));
        assert_eq!(calls.created.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn a_missing_password_is_a_credential_error() {
        let (handler, _calls) = handler();
        let repository = MemoryRepository::default();
        repository.add("a", "A", Some("pw"));
        repository.forget_password("a");

        assert!(matches!(handler.open(&repository, "a").await, Err(DriverError::CredentialsUnavailable(_))));
        assert!(!handler.is_pooled("a").await);
    }

    #[tokio::test]
    async fn a_driver_that_fails_to_connect_is_not_pooled() {
        let (handler, _calls) = handler();
        let repository = MemoryRepository::default();
        repository.add("a", "refuses", None);

        assert!(matches!(handler.get_or_connect(&repository, "a").await, Err(DriverError::UriMalformed(_))));
        assert!(!handler.is_pooled("a").await);
    }

    #[tokio::test]
    async fn the_password_reaches_the_driver() {
        let (handler, _calls) = handler();
        let repository = MemoryRepository::default();
        repository.add("a", "A", Some("pw"));

        let driver = handler.get_or_connect(&repository, "a").await.unwrap();
        assert!(driver.list_databases().await.is_ok());
    }
}
