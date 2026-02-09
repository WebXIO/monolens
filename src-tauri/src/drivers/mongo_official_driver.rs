use async_trait::async_trait;
use mongodb::{bson::doc, Client};
use crate::{connector::models::connection::Connection, drivers::{driver::{DatabaseDriver, TestStage}, errors::DriverError}};


pub struct MongoDbOfficialDriver {
   client: Option<Client>,
   connection: Connection,
}


impl MongoDbOfficialDriver {
   pub fn new(connection: Connection) -> Self {
      MongoDbOfficialDriver { client: None, connection }
   }
}

#[async_trait]
impl DatabaseDriver for MongoDbOfficialDriver {

   async fn connect(&mut self) -> Result<(), DriverError> {
      if self.client.is_none() {
         self.client = Some(
            Client::with_uri_str(self.connection.uri.to_string())
               .await
               .map_err(|e| DriverError::ConnectionFailed(e.to_string()))?
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
   /// 1. Initialize Connection
   /// 2. Ping Database
   /// 3. Reading Server stats
   /// 4. Version detection
   /// 5. Connected
   async fn test_connection(&self) -> Result<Vec<TestStage>, DriverError> {

      let mut stages: Vec<TestStage> = vec![
         TestStage::new(None, String::from("Initialize Connection")),
         TestStage::new(None, String::from("Ping Database")),
         TestStage::new(None, String::from("Reading Server status")),
         TestStage::new(None, String::from("Detecting Mongodb version")),
         TestStage::new(None, String::from("Connected")),
      ];

      // Stage 0: Initialize Connection
      let client_ref = self.client.as_ref().ok_or(DriverError::ClientNotInitialized)?;
      stages[0].status = Some(true);

      let default_database_name = self.connection.authentication.database
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

      // Stage 2: Reading Server status
      let server_status = default_database
         .run_command(doc! {"serverStatus": 1})
         .await
         .map_err(|e| DriverError::ServerStatusFailed(e.to_string()))?;
      stages[2].status = Some(true);

      // Stage 3: Detecting MongoDB version
      if server_status.get_str("version").is_ok() {
         stages[3].status = Some(true);
      } else {
         stages[3].status = Some(false);
      }

      // Stage 4: Connected
      stages[4].status = Some(true);

      Ok(stages)
   }
}