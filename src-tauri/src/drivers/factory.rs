use crate::{connector::models::connection::{Connection, ConnectorKind}, drivers::{driver::DatabaseDriver, mongo_official_driver::MongoDbOfficialDriver}};

struct DatabaseDriverFactory;

impl DatabaseDriverFactory {
   pub fn create(connection: Connection) -> Result<Box<dyn DatabaseDriver>, &'static str> {
      match connection.connector {
         ConnectorKind::MongoDb => Ok(Box::new(MongoDbOfficialDriver::new(connection))),
         _ => Err("Unknown Driver detected")
      }
   }
}