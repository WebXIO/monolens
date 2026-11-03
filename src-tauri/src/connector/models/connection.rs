use serde::{Deserialize, Serialize};

use crate::connector::models::{
    authentication::AuthenticationOptions, create_connection::CreateConnection,
    options::ConnectionOptions,
};
use uuid::Uuid;

#[derive(Serialize, Debug, Deserialize, Clone, PartialEq)]
pub enum ConnectorKind {
    MongoDb,
}

/// Where the server is: host and port with option fields, or a connection string.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "kind")]
pub enum ConnectionTarget {
    Host { host: String, port: u16 },
    /// `mongodb://` or `mongodb+srv://`, stored without user info
    Uri { uri: String },
}

#[derive(Serialize, Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Connection {
    pub id: String,
    pub name: String,
    pub connector: ConnectorKind,
    pub target: ConnectionTarget,
    pub authentication: AuthenticationOptions,
    #[serde(default)]
    pub options: ConnectionOptions,
}

impl From<CreateConnection> for Connection {
    fn from(entity: CreateConnection) -> Self {
        Connection {
            id: Uuid::new_v4().to_string(),
            name: entity.name,
            connector: entity.connector,
            target: entity.target,
            authentication: entity.authentication,
            options: entity.options,
        }
    }
}
