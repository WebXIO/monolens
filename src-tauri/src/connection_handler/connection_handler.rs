use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;
use crate::{connector::models::connection::Connection, drivers::{driver::DatabaseDriver, factory::DatabaseDriverFactory}};


pub struct ConnectionHandler {
   connection_pool: Mutex<HashMap<String, Arc<Mutex<Box<dyn DatabaseDriver>>>>>
}

impl ConnectionHandler {
   pub fn new() -> Self {
      ConnectionHandler { connection_pool: Mutex::new(HashMap::new()) }
   }


   pub async fn get_or_connect(&self, connection: Connection) -> Result<Arc<Mutex<Box<dyn DatabaseDriver>>>, &'static str> {
      let mut pool = self.connection_pool.lock().await;

      if let Some(driver) = pool.get(&connection.id) {
         return Ok(Arc::clone(driver));
      }

      let mut driver = DatabaseDriverFactory::create(connection.clone()).map_err(|e| format!("Failed to create driver {}", e)).unwrap();

      driver.connect().await.map_err(|e| format!("Failed to connect: {}", e)).unwrap();

      let shared_driver = Arc::from(Mutex::new(driver));

      pool.insert(connection.id.clone(), Arc::clone(&shared_driver));

      return Ok(shared_driver)
   }

   pub async fn disconnect(&self, connection_id: &str) -> Result<(), &str> {
      let mut pool = self.connection_pool.lock().await;

      if let Some(lock) = pool.remove(connection_id) {
         let mut driver = lock.lock().await;
         driver.disconnect().await.map_err(|e| format!("Failed to disconnect: {}", e)).unwrap();
      }

      Ok(())
   }
}