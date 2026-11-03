use crate::{
    connector::models::connection::{Connection, ConnectorKind},
    drivers::{
        driver::DatabaseDriver, errors::DriverError, mongo_official_driver::MongoDbOfficialDriver,
    },
};

pub struct DatabaseDriverFactory;

impl DatabaseDriverFactory {
    pub fn create(connection: Connection) -> Result<Box<dyn DatabaseDriver>, DriverError> {
        match connection.connector {
            ConnectorKind::MongoDb => Ok(Box::new(MongoDbOfficialDriver::new(connection))),
        }
    }

    /// A driver for the connection test: it gives up sooner when no server answers.
    pub fn create_for_test(connection: Connection) -> Result<Box<dyn DatabaseDriver>, DriverError> {
        match connection.connector {
            ConnectorKind::MongoDb => Ok(Box::new(MongoDbOfficialDriver::for_test(connection))),
        }
    }
}
