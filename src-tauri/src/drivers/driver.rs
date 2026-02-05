use async_trait::async_trait;

#[async_trait]
pub trait DatabaseDriver: Send + Sync {
   async fn connect(&mut self) -> Result<(), &'static str>;
   async fn test_connection(&self) -> Result<bool, &'static str>;
}