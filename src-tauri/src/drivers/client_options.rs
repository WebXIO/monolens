use std::path::PathBuf;
use std::time::Duration;

use mongodb::options::{ClientOptions, Credential, ServerAddress, Tls, TlsOptions};
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};

use crate::connector::models::{
    authentication::{AuthenticationKind, AuthenticationOptions},
    connection::{Connection, ConnectionTarget},
    options::ConnectionOptions,
};
use crate::drivers::errors::DriverError;

const APP_NAME: &str = "Monolens";
const DEFAULT_AUTH_SOURCE: &str = "admin";

/// The driver's options for a connection, without building a URL string by hand.
/// A connection string is parsed by the driver (an srv string resolves DNS here).
pub async fn client_options(connection: &Connection) -> Result<ClientOptions, DriverError> {
    let mut options = match &connection.target {
        ConnectionTarget::Host { host, port } => host_options(host, *port, &connection.options),
        ConnectionTarget::Uri { uri } => {
            let full = with_user_info(uri, &connection.authentication);
            ClientOptions::parse(&full)
                .await
                .map_err(|e| DriverError::UriMalformed(e.to_string()))?
        }
    };

    options.app_name = Some(APP_NAME.to_string());
    options.credential = credential(&connection.authentication, options.credential.take());

    Ok(options)
}

/// Stored connection strings carry no credentials. The driver validates some options against
/// them (an `authMechanism` needs a username), so they go back in, encoded, right before parsing.
fn with_user_info(uri: &str, auth: &AuthenticationOptions) -> String {
    let Some(username) = non_empty(&auth.username) else {
        return uri.to_string();
    };
    if auth.kind != AuthenticationKind::BASIC {
        return uri.to_string();
    }
    let Some(scheme_end) = uri.find("://").map(|i| i + 3) else {
        return uri.to_string();
    };

    let mut user_info = utf8_percent_encode(&username, NON_ALPHANUMERIC).to_string();
    if let Some(password) = &auth.password {
        user_info.push(':');
        user_info.push_str(&utf8_percent_encode(password, NON_ALPHANUMERIC).to_string());
    }

    format!("{}{}@{}", &uri[..scheme_end], user_info, &uri[scheme_end..])
}

fn non_empty(value: &Option<String>) -> Option<String> {
    value.as_ref().map(|v| v.trim()).filter(|v| !v.is_empty()).map(String::from)
}

fn host_options(host: &str, port: u16, settings: &ConnectionOptions) -> ClientOptions {
    let mut options = ClientOptions::builder()
        .hosts(vec![ServerAddress::Tcp {
            host: host.to_string(),
            port: Some(port),
        }])
        .build();

    if settings.tls {
        options.tls = Some(Tls::Enabled(
            TlsOptions::builder()
                .ca_file_path(non_empty(&settings.tls_ca_file).map(PathBuf::from))
                .allow_invalid_certificates(settings.tls_allow_invalid_certificates)
                .build(),
        ));
    }

    options.repl_set_name = non_empty(&settings.replica_set);
    options.server_selection_timeout = settings
        .server_selection_timeout_secs
        .map(|secs| Duration::from_secs(u64::from(secs)));

    options
}

/// The authentication database is the form's, else the string's `authSource`, else admin.
/// A mechanism set in the connection string stays.
fn credential(auth: &AuthenticationOptions, from_uri: Option<Credential>) -> Option<Credential> {
    match auth.kind {
        AuthenticationKind::NONE => None,
        AuthenticationKind::BASIC => {
            let source = non_empty(&auth.database)
                .or_else(|| from_uri.as_ref().and_then(|c| c.source.clone()))
                .unwrap_or_else(|| DEFAULT_AUTH_SOURCE.to_string());

            let mut credential = from_uri.unwrap_or_default();
            credential.username = auth.username.clone();
            credential.password = auth.password.clone();
            credential.source = Some(source);
            Some(credential)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connector::models::connection::ConnectorKind;
    use mongodb::options::AuthMechanism;

    fn connection(target: ConnectionTarget, authentication: AuthenticationOptions, options: ConnectionOptions) -> Connection {
        Connection {
            id: "id".into(),
            name: "test".into(),
            connector: ConnectorKind::MongoDb,
            target,
            authentication,
            options,
        }
    }

    fn host() -> ConnectionTarget {
        ConnectionTarget::Host { host: "db.local".into(), port: 27018 }
    }

    fn uri(value: &str) -> ConnectionTarget {
        ConnectionTarget::Uri { uri: value.into() }
    }

    fn basic(database: Option<&str>) -> AuthenticationOptions {
        AuthenticationOptions {
            kind: AuthenticationKind::BASIC,
            username: Some("ada".into()),
            password: Some("p@ss:w/rd".into()),
            database: database.map(String::from),
        }
    }

    #[tokio::test]
    async fn host_mode_uses_host_and_port_and_driver_defaults() {
        let options = client_options(&connection(host(), AuthenticationOptions::default(), ConnectionOptions::default()))
            .await
            .unwrap();

        assert_eq!(options.hosts, vec![ServerAddress::Tcp { host: "db.local".into(), port: Some(27018) }]);
        assert_eq!(options.app_name.as_deref(), Some("Monolens"));
        assert!(options.credential.is_none());
        assert!(options.tls.is_none());
        assert!(options.repl_set_name.is_none());
        assert!(options.server_selection_timeout.is_none());
    }

    #[tokio::test]
    async fn host_mode_applies_tls_replica_set_and_timeout() {
        let settings = ConnectionOptions {
            tls: true,
            tls_ca_file: Some("/etc/ssl/ca.pem".into()),
            tls_allow_invalid_certificates: true,
            replica_set: Some("rs0".into()),
            server_selection_timeout_secs: Some(5),
        };

        let options = client_options(&connection(host(), AuthenticationOptions::default(), settings)).await.unwrap();

        match options.tls {
            Some(Tls::Enabled(tls)) => {
                assert_eq!(tls.ca_file_path, Some(PathBuf::from("/etc/ssl/ca.pem")));
                assert_eq!(tls.allow_invalid_certificates, Some(true));
            }
            other => panic!("expected tls, got {other:?}"),
        }
        assert_eq!(options.repl_set_name.as_deref(), Some("rs0"));
        assert_eq!(options.server_selection_timeout, Some(Duration::from_secs(5)));
    }

    #[tokio::test]
    async fn host_mode_ignores_blank_option_fields() {
        let settings = ConnectionOptions {
            tls: true,
            tls_ca_file: Some("  ".into()),
            replica_set: Some("".into()),
            ..ConnectionOptions::default()
        };

        let options = client_options(&connection(host(), AuthenticationOptions::default(), settings)).await.unwrap();

        match options.tls {
            Some(Tls::Enabled(tls)) => assert_eq!(tls.ca_file_path, None),
            other => panic!("expected tls, got {other:?}"),
        }
        assert!(options.repl_set_name.is_none());
    }

    #[tokio::test]
    async fn basic_authentication_sets_the_credential_unencoded() {
        let options = client_options(&connection(host(), basic(Some("shop")), ConnectionOptions::default()))
            .await
            .unwrap();

        let credential = options.credential.unwrap();
        assert_eq!(credential.username.as_deref(), Some("ada"));
        assert_eq!(credential.password.as_deref(), Some("p@ss:w/rd"));
        assert_eq!(credential.source.as_deref(), Some("shop"));
    }

    #[tokio::test]
    async fn the_authentication_database_defaults_to_admin() {
        let options = client_options(&connection(host(), basic(Some(" ")), ConnectionOptions::default()))
            .await
            .unwrap();
        assert_eq!(options.credential.unwrap().source.as_deref(), Some("admin"));
    }

    #[tokio::test]
    async fn uri_mode_parses_hosts_and_options_from_the_string() {
        let options = client_options(&connection(
            uri("mongodb://h1:27017,h2:27018/?replicaSet=rs0&tls=true&serverSelectionTimeoutMS=2000"),
            AuthenticationOptions::default(),
            // ignored for a connection string
            ConnectionOptions { replica_set: Some("other".into()), ..ConnectionOptions::default() },
        ))
        .await
        .unwrap();

        assert_eq!(options.hosts.len(), 2);
        assert_eq!(options.repl_set_name.as_deref(), Some("rs0"));
        assert!(matches!(options.tls, Some(Tls::Enabled(_))));
        assert_eq!(options.server_selection_timeout, Some(Duration::from_millis(2000)));
        assert!(options.credential.is_none());
    }

    #[tokio::test]
    async fn uri_mode_keeps_auth_source_and_mechanism_of_the_string() {
        let options = client_options(&connection(
            uri("mongodb://h/?authSource=shop&authMechanism=SCRAM-SHA-256"),
            basic(None),
            ConnectionOptions::default(),
        ))
        .await
        .unwrap();

        let credential = options.credential.unwrap();
        assert_eq!(credential.source.as_deref(), Some("shop"));
        assert_eq!(credential.mechanism, Some(AuthMechanism::ScramSha256));
        assert_eq!(credential.username.as_deref(), Some("ada"));
    }

    #[tokio::test]
    async fn the_form_authentication_database_wins_over_the_string() {
        let options = client_options(&connection(uri("mongodb://h/?authSource=shop"), basic(Some("admin")), ConnectionOptions::default()))
            .await
            .unwrap();
        assert_eq!(options.credential.unwrap().source.as_deref(), Some("admin"));
    }

    #[tokio::test]
    async fn no_authentication_drops_a_credential_of_the_string() {
        let options = client_options(&connection(uri("mongodb://h/?authSource=shop"), AuthenticationOptions::default(), ConnectionOptions::default()))
            .await
            .unwrap();
        assert!(options.credential.is_none());
    }

    #[tokio::test]
    async fn uri_mode_round_trips_credentials_with_special_characters() {
        let options = client_options(&connection(uri("mongodb://h1,h2/shop"), basic(None), ConnectionOptions::default()))
            .await
            .unwrap();

        let credential = options.credential.unwrap();
        assert_eq!(credential.username.as_deref(), Some("ada"));
        assert_eq!(credential.password.as_deref(), Some("p@ss:w/rd"));
        assert_eq!(credential.source.as_deref(), Some("shop"));
    }

    #[test]
    fn user_info_is_only_added_for_basic_authentication_with_a_username() {
        assert_eq!(with_user_info("mongodb+srv://c.example.net/?x=1", &basic(None)), "mongodb+srv://ada:p%40ss%3Aw%2Frd@c.example.net/?x=1");
        assert_eq!(with_user_info("mongodb://h", &AuthenticationOptions::default()), "mongodb://h");
        let no_user = AuthenticationOptions { username: Some(" ".into()), ..basic(None) };
        assert_eq!(with_user_info("mongodb://h", &no_user), "mongodb://h");
        let no_password = AuthenticationOptions { password: None, ..basic(None) };
        assert_eq!(with_user_info("mongodb://h", &no_password), "mongodb://ada@h");
    }

    #[tokio::test]
    async fn a_malformed_string_is_a_uri_error() {
        let result = client_options(&connection(uri("mongodb://h/?replicaSet"), AuthenticationOptions::default(), ConnectionOptions::default())).await;
        assert!(matches!(result, Err(DriverError::UriMalformed(_))));
    }
}
