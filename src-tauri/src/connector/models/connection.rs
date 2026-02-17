use serde::{Deserialize, Serialize};

use crate::connector::models::{
    authentication::AuthenticationOptions, connection_type::ConnectionType, create_connection::CreateConnection,
};
use uuid::Uuid;

#[derive(Serialize, Debug, Deserialize, Clone)]
pub enum ConnectorKind {
    MongoDb,
}

#[derive(Serialize, Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Connection {
    pub id: String,
    pub name: String,
    pub uri: String,
    pub port: u16,
    pub connector: ConnectorKind,
    pub connection_type: ConnectionType,
    pub authentication: AuthenticationOptions,
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
            authentication: AuthenticationOptions::default(),
        }
    }
}

impl From<CreateConnection> for Connection {
    fn from(entity: CreateConnection) -> Self {
        Connection {
            id: Uuid::new_v4().to_string(),
            name: entity.name,
            uri: entity.uri,
            authentication: entity.authentication,
            connection_type: entity.connection_type,
            connector: entity.connector,
            port: entity.port
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
            authentication: value.authentication,
        }
    }
}
