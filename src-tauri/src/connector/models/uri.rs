use percent_encoding::percent_decode_str;

const SCHEMES: [&str; 2] = ["mongodb://", "mongodb+srv://"];

/// A connection string split into the string without user info and the credentials it carried.
#[derive(Debug, PartialEq)]
pub struct UriCredentials {
    pub uri: String,
    pub username: Option<String>,
    pub password: Option<String>,
}

pub fn is_connection_string(value: &str) -> bool {
    SCHEMES.iter().any(|scheme| value.starts_with(scheme))
}

/// Takes `user:password@` out of a connection string and percent-decodes both.
/// The hosts, the path and the options stay as they are.
pub fn split_uri_credentials(value: &str) -> Result<UriCredentials, String> {
    let scheme = SCHEMES
        .iter()
        .find(|scheme| value.starts_with(*scheme))
        .ok_or_else(|| String::from("A connection string starts with mongodb:// or mongodb+srv://"))?;

    let rest = &value[scheme.len()..];
    let authority_end = rest.find(['/', '?']).unwrap_or(rest.len());
    let authority = &rest[..authority_end];

    // the last @ ends the user info, an @ in a password must be percent-encoded anyway
    let (user_info, hosts) = match authority.rfind('@') {
        Some(at) => (Some(&authority[..at]), &authority[at + 1..]),
        None => (None, authority),
    };

    if hosts.is_empty() {
        return Err(String::from("The connection string names no host"));
    }

    let decode = |part: &str| {
        percent_decode_str(part)
            .decode_utf8()
            .map(|decoded| decoded.into_owned())
            .map_err(|_| String::from("The credentials in the connection string are not valid UTF-8"))
    };

    let (username, password) = match user_info {
        None => (None, None),
        Some(info) => match info.split_once(':') {
            Some((user, pass)) => (Some(decode(user)?), Some(decode(pass)?)),
            None => (Some(decode(info)?), None),
        },
    };

    Ok(UriCredentials {
        uri: format!("{scheme}{hosts}{}", &rest[authority_end..]),
        username: username.filter(|u| !u.is_empty()),
        password,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognises_both_schemes() {
        assert!(is_connection_string("mongodb://localhost"));
        assert!(is_connection_string("mongodb+srv://cluster0.example.net"));
        assert!(!is_connection_string("localhost"));
        assert!(!is_connection_string("http://localhost"));
        assert!(!is_connection_string(""));
    }

    #[test]
    fn leaves_a_string_without_user_info_alone() {
        let split = split_uri_credentials("mongodb://localhost:27017/?replicaSet=rs0").unwrap();
        assert_eq!(
            split,
            UriCredentials { uri: "mongodb://localhost:27017/?replicaSet=rs0".into(), username: None, password: None }
        );
    }

    #[test]
    fn takes_user_and_password_out() {
        let split = split_uri_credentials("mongodb+srv://ada:s3cr3t@cluster0.example.net/shop?retryWrites=true").unwrap();
        assert_eq!(split.uri, "mongodb+srv://cluster0.example.net/shop?retryWrites=true");
        assert_eq!(split.username.as_deref(), Some("ada"));
        assert_eq!(split.password.as_deref(), Some("s3cr3t"));
    }

    #[test]
    fn decodes_percent_encoded_credentials() {
        let split = split_uri_credentials("mongodb://us%40er:p%3Ass%40w%2Frd@db1:27017,db2:27018/?authSource=admin").unwrap();
        assert_eq!(split.uri, "mongodb://db1:27017,db2:27018/?authSource=admin");
        assert_eq!(split.username.as_deref(), Some("us@er"));
        assert_eq!(split.password.as_deref(), Some("p:ss@w/rd"));
    }

    #[test]
    fn takes_a_user_without_password_out() {
        let split = split_uri_credentials("mongodb://ada@localhost").unwrap();
        assert_eq!(split.uri, "mongodb://localhost");
        assert_eq!(split.username.as_deref(), Some("ada"));
        assert_eq!(split.password, None);
    }

    #[test]
    fn keeps_an_empty_password() {
        let split = split_uri_credentials("mongodb://ada:@localhost").unwrap();
        assert_eq!(split.password.as_deref(), Some(""));
    }

    #[test]
    fn handles_a_string_that_ends_after_the_host() {
        let split = split_uri_credentials("mongodb://u:p@localhost").unwrap();
        assert_eq!(split.uri, "mongodb://localhost");
    }

    #[test]
    fn an_at_in_the_options_is_not_user_info() {
        let split = split_uri_credentials("mongodb://localhost/?appName=a@b").unwrap();
        assert_eq!(split.uri, "mongodb://localhost/?appName=a@b");
        assert_eq!(split.username, None);
    }

    #[test]
    fn refuses_other_schemes_and_missing_hosts() {
        assert!(split_uri_credentials("localhost:27017").is_err());
        assert!(split_uri_credentials("mongodb://").is_err());
        assert!(split_uri_credentials("mongodb://user:pw@/db").is_err());
    }

    #[test]
    fn refuses_credentials_that_are_not_utf8() {
        assert!(split_uri_credentials("mongodb://%FF:pw@localhost").is_err());
    }
}
