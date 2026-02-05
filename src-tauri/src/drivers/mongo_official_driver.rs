use async_trait::async_trait;
use mongodb::Client;
use tracing::Instrument;

use crate::{connector::models::connection::Connection, drivers::driver::DatabaseDriver};


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

   async fn connect(&mut self) -> Result<(), &'static str> {
      if !self.client.is_some() {
         self.client = Some(Client::with_uri_str(self.connection.uri.to_string()).await.map_err(|_| "Failed to connect to MongoDB")?);
      }

      Ok(())
   }

   async fn test_connection(&self) -> Result<bool, &'static str> {
      let client_ref = self.client.as_ref().ok_or("Client could not connect")?;

      client_ref.list_databases().await.map_err(|_| "Failed to list database")?;

      return Ok(true);
   }
}