use serde::{Deserialize, Serialize};

use crate::connector::models::{authentication::AuthenticationOptions, connection_type::ConnectionType};
use uuid::Uuid;

#[derive(Serialize, Debug, Deserialize, Clone)]
pub enum ConnectorKind {
    MongoDb
}

#[derive(Serialize, Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Connection {
   pub id: String,
   pub name: String,
   pub uri: String,
   pub port: u64,
   pub connector: ConnectorKind,
   pub connection_type: ConnectionType,
   pub authentication: AuthenticationOptions
}


impl Default for Connection {
    fn default() -> Self {
        Connection {
         id: Uuid::new_v4().to_string(),
         name: String::new(),
         uri: String::new(),
         port: 27017,
         connector: ConnectorKind::MongoDb,
         connection_type: ConnectionType::Standalone,
         authentication: AuthenticationOptions::default()
      }
    }
}

impl Connection {
    pub fn new(value: Connection) -> Self {
        Connection {
         id: Uuid::new_v4().to_string(),
         name: value.name,
         uri: value.uri,
         port: value.port,
         connector: value.connector,
         connection_type: value.connection_type,
         authentication: value.authentication
      }
    }
}