use crate::{connector::models::connection::Connection, drivers::{driver::TestStage, errors::DriverError, factory::DatabaseDriverFactory}};

#[tauri::command]
pub async fn test_connection(connection: Connection) -> Result<Vec<TestStage>, DriverError> {
   let mut driver = DatabaseDriverFactory::create(connection).unwrap();

   driver.connect().await?;

   let result = driver.test_connection().await?;

   Ok(result)
}