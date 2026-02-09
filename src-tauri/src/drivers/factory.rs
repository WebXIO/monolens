use crate::{connector::models::connection::{Connection, ConnectorKind}, drivers::{driver::DatabaseDriver, errors::DriverError, mongo_official_driver::MongoDbOfficialDriver}};

pub struct DatabaseDriverFactory;

impl DatabaseDriverFactory {
   pub fn create(connection: Connection) -> Result<Box<dyn DatabaseDriver>, DriverError> {
      match connection.connector {
         ConnectorKind::MongoDb => Ok(Box::new(MongoDbOfficialDriver::new(connection))),
      }
   }
}