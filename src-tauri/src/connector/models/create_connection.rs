use serde::{Deserialize, Serialize};

use crate::connector::models::{
    authentication::AuthenticationOptions,
    connection::{ConnectionTarget, ConnectorKind},
    options::ConnectionOptions,
};

#[derive(Serialize, Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CreateConnection {
    pub name: String,
    pub connector: ConnectorKind,
    pub target: ConnectionTarget,
    pub authentication: AuthenticationOptions,
    #[serde(default)]
    pub options: ConnectionOptions,
}
