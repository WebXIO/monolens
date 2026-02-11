use async_trait::async_trait;
use mongodb::{bson::doc, Client};
use crate::{connector::models::{authentication::AuthenticationKind, connection::Connection}, drivers::{driver::{DatabaseDriver, TestStage}, errors::DriverError}};


pub struct MongoDbOfficialDriver {
   client: Option<Client>,
   connection: Connection,
}


impl MongoDbOfficialDriver {
   pub fn new(connection: Connection) -> Self {
      MongoDbOfficialDriver { client: None, connection }
   }

   fn build_connection_string(&self) -> String {
      let conn = &self.connection;
      
      // If the URI already looks like a full MongoDB connection string, use it directly
      if conn.uri.starts_with("mongodb://") || conn.uri.starts_with("mongodb+srv://") {
         return conn.uri.clone();
      }
      
      // Build connection string from parts
      let auth_part = match &conn.authentication.kind {
         AuthenticationKind::BASIC => {
            let username = conn.authentication.username.as_deref().unwrap_or("");
            let password = conn.authentication.password.as_deref().unwrap_or("");
            if !username.is_empty() {
               format!("{}:{}@", username, password)
            } else {
               String::new()
            }
         },
         AuthenticationKind::NONE => String::new(),
      };
      
      let auth_db = conn.authentication.database.as_deref().unwrap_or("admin");
      
      format!("mongodb://{}{}:{}/?authSource={}", auth_part, conn.uri, conn.port, auth_db)
   }
}

#[async_trait]
impl DatabaseDriver for MongoDbOfficialDriver {

   async fn connect(&mut self) -> Result<(), DriverError> {
      if self.client.is_none() {
         let connection_string = self.build_connection_string();
         self.client = Some(
            Client::with_uri_str(&connection_string)
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

   async fn list_databases(&self) -> Result<Vec<String>, DriverError> {
      let client = self.client.as_ref().ok_or(DriverError::ClientNotInitialized)?;
      
      client
         .list_database_names()
         .await
         .map_err(|e| DriverError::ListDatabasesFailed(e.to_string()))
   }
}