use mongodb::{Client, results::DatabaseSpecification};

#[derive(Debug)]
pub struct Connection {
   uri: String
}

impl Connection {
   pub fn new(uri: &str ) -> Connection {
      Connection {
         uri: uri.to_string()
      }
   }

   pub async fn get_databases(&self) -> Vec<DatabaseSpecification> {
      let client = Client::with_uri_str(self.uri.as_str()).await.unwrap();

      let databases = client.list_databases().await.unwrap();

      return databases;
   }
}