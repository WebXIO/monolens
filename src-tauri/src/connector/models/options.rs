use serde::{Deserialize, Serialize};

/// Settings for a connection by host and port. A connection string carries its own.
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct ConnectionOptions {
    pub tls: bool,
    pub tls_ca_file: Option<String>,
    pub tls_allow_invalid_certificates: bool,
    pub replica_set: Option<String>,
    /// `None` means the driver's default (30 s)
    pub server_selection_timeout_secs: Option<u32>,
}
