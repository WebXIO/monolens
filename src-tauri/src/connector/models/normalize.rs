use crate::connector::models::{
    authentication::{AuthenticationKind, AuthenticationOptions},
    connection::ConnectionTarget,
    uri::{is_connection_string, split_uri_credentials},
};

/// `db.local:27018` typed into the host field: the host and the port after it.
pub fn split_host_port(host: &str) -> Option<(&str, u16)> {
    let (name, port) = host.rsplit_once(':')?;
    // an IPv6 address has more colons and stays as it is
    if name.is_empty() || name.contains(':') {
        return None;
    }
    port.parse().ok().map(|port| (name, port))
}

/// Checks a connection before it is stored and moves credentials out of a connection string:
/// the username into the authentication, the password to where the repository keeps passwords.
/// Credentials typed into the form win over the ones in the string.
pub fn normalize(
    name: &str,
    target: ConnectionTarget,
    mut auth: AuthenticationOptions,
) -> Result<(ConnectionTarget, AuthenticationOptions), String> {
    if name.trim().is_empty() {
        return Err(String::from("The connection needs a name."));
    }

    let target = match target {
        ConnectionTarget::Host { host, port } => {
            let host = host.trim().to_string();
            if host.is_empty() {
                return Err(String::from("The connection needs a host."));
            }
            if is_connection_string(&host) || host.contains("://") {
                return Err(String::from(
                    "Enter mongodb:// and mongodb+srv:// addresses as a connection string, not as a host.",
                ));
            }
            if split_host_port(&host).is_some() {
                return Err(String::from("Enter the port in the port field, not after the host."));
            }
            if port == 0 {
                return Err(String::from("The port must be between 1 and 65535."));
            }
            ConnectionTarget::Host { host, port }
        }
        ConnectionTarget::Uri { uri } => {
            let split = split_uri_credentials(uri.trim())?;
            if split.username.is_some() || split.password.is_some() {
                auth.kind = AuthenticationKind::BASIC;
            }
            if auth.username.as_deref().unwrap_or("").is_empty() {
                if let Some(username) = split.username {
                    auth.username = Some(username);
                }
            }
            if auth.password.is_none() {
                auth.password = split.password;
            }
            ConnectionTarget::Uri { uri: split.uri }
        }
    };

    if auth.kind == AuthenticationKind::BASIC
        && auth.username.as_deref().unwrap_or("").trim().is_empty()
    {
        return Err(String::from("Authentication needs a username."));
    }

    Ok((target, auth))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn none() -> AuthenticationOptions {
        AuthenticationOptions::default()
    }

    fn basic(username: &str, password: Option<&str>) -> AuthenticationOptions {
        AuthenticationOptions {
            kind: AuthenticationKind::BASIC,
            username: Some(username.into()),
            password: password.map(String::from),
            database: None,
        }
    }

    fn uri(value: &str) -> ConnectionTarget {
        ConnectionTarget::Uri { uri: value.into() }
    }

    #[test]
    fn trims_a_host() {
        let (target, _) = normalize("Local", ConnectionTarget::Host { host: " localhost ".into(), port: 27017 }, none()).unwrap();
        assert_eq!(target, ConnectionTarget::Host { host: "localhost".into(), port: 27017 });
    }

    #[test]
    fn refuses_a_missing_name_host_or_port() {
        assert!(normalize(" ", ConnectionTarget::Host { host: "h".into(), port: 1 }, none()).is_err());
        assert!(normalize("n", ConnectionTarget::Host { host: " ".into(), port: 1 }, none()).is_err());
        assert!(normalize("n", ConnectionTarget::Host { host: "h".into(), port: 0 }, none()).is_err());
    }

    #[test]
    fn refuses_a_connection_string_in_the_host_field() {
        let err = normalize("n", ConnectionTarget::Host { host: "mongodb+srv://c.example.net".into(), port: 27017 }, none()).unwrap_err();
        assert!(err.contains("connection string"));
    }

    #[test]
    fn refuses_a_port_after_the_host() {
        let err = normalize("n", ConnectionTarget::Host { host: "db.local:27018".into(), port: 27017 }, none()).unwrap_err();
        assert!(err.contains("port field"));
    }

    #[test]
    fn splits_host_and_port_but_leaves_ipv6_and_names_alone() {
        assert_eq!(split_host_port("db.local:27018"), Some(("db.local", 27018)));
        assert_eq!(split_host_port("db.local"), None);
        assert_eq!(split_host_port("::1"), None);
        assert_eq!(split_host_port("fe80::1:27017"), None);
        assert_eq!(split_host_port(":27017"), None);
        assert_eq!(split_host_port("h:port"), None);
    }

    #[test]
    fn refuses_basic_authentication_without_username() {
        assert!(normalize("n", ConnectionTarget::Host { host: "h".into(), port: 1 }, basic(" ", Some("pw"))).is_err());
    }

    #[test]
    fn moves_credentials_out_of_a_connection_string() {
        let (target, auth) = normalize("n", uri("mongodb://ada:pw@h1,h2/?replicaSet=rs"), none()).unwrap();
        assert_eq!(target, uri("mongodb://h1,h2/?replicaSet=rs"));
        assert_eq!(auth.kind, AuthenticationKind::BASIC);
        assert_eq!(auth.username.as_deref(), Some("ada"));
        assert_eq!(auth.password.as_deref(), Some("pw"));
    }

    #[test]
    fn typed_credentials_win_over_the_string() {
        let (_, auth) = normalize("n", uri("mongodb://ada:pw@h"), basic("bob", Some("typed"))).unwrap();
        assert_eq!(auth.username.as_deref(), Some("bob"));
        assert_eq!(auth.password.as_deref(), Some("typed"));
    }

    #[test]
    fn a_string_without_credentials_keeps_the_authentication() {
        let (_, auth) = normalize("n", uri("mongodb://h"), none()).unwrap();
        assert_eq!(auth, none());
    }

    #[test]
    fn refuses_a_string_with_another_scheme() {
        assert!(normalize("n", uri("http://h"), none()).is_err());
    }
}
