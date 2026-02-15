use serde::{Deserialize, Serialize};

use crate::connector::models::{authentication::AuthenticationOptions, connection::ConnectorKind, connection_type::ConnectionType};


#[derive(Serialize, Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CreateConnection {
    pub name: String,
    pub uri: String,
    pub port: u16,
    pub connector: ConnectorKind,
    pub connection_type: ConnectionType,
    pub authentication: AuthenticationOptions,
}