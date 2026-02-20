use crate::{
    connector::models::connection::Connection,
    drivers::{driver::DatabaseDriver, errors::DriverError, factory::DatabaseDriverFactory},
};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

pub struct ConnectionHandler {
    connection_pool: Mutex<HashMap<String, Arc<Mutex<Box<dyn DatabaseDriver>>>>>,
}

impl ConnectionHandler {
    pub fn new() -> Self {
        ConnectionHandler {
            connection_pool: Mutex::new(HashMap::new()),
        }
    }

    pub async fn get_or_connect(
        &self,
        connection: Connection,
    ) -> Result<Arc<Mutex<Box<dyn DatabaseDriver>>>, DriverError> {
        let mut pool = self.connection_pool.lock().await;

        if let Some(driver) = pool.get(&connection.id) {
            return Ok(Arc::clone(driver));
        }

        let mut driver = DatabaseDriverFactory::create(connection.clone())?;

        driver.connect().await?;

        let shared_driver = Arc::from(Mutex::new(driver));

        pool.insert(connection.id.clone(), Arc::clone(&shared_driver));

        return Ok(shared_driver);
    }

    pub async fn disconnect(&self, connection_id: &str) -> Result<(), DriverError> {
        let mut pool = self.connection_pool.lock().await;

        if let Some(lock) = pool.remove(connection_id) {
            let mut driver = lock.lock().await;
            driver.disconnect().await?;
        }

        Ok(())
    }
}
