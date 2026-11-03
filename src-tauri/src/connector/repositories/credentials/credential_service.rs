use async_trait::async_trait;
use keyring::Entry;

use crate::connector::commands::error::CredentialError;
use crate::connector::repositories::credentials::credential_store::CredentialStore;

/// Passwords in the OS keyring: Keychain, Credential Manager or Secret Service.
pub struct CredentialService {
    service_name: String,
}

impl CredentialService {
    pub fn new(service_name: impl Into<String>) -> Self {
        Self {
            service_name: service_name.into(),
        }
    }

    fn get_key(&self, connection_id: &str) -> String {
        format!("connection:{}:basic_password", connection_id)
    }

    fn not_found(connection_id: &str, err: keyring::Error) -> CredentialError {
        match err {
            keyring::Error::NoEntry => CredentialError::NotFound(connection_id.to_string()),
            other => CredentialError::Keyring(other),
        }
    }
}

#[async_trait]
impl CredentialStore for CredentialService {
    async fn set_password(
        &self,
        connection_id: &str,
        password: &str,
    ) -> Result<(), CredentialError> {
        let service = self.service_name.clone();
        let key = self.get_key(connection_id);
        let pass = password.to_string();

        // keyring calls block on the platform store
        tokio::task::spawn_blocking(move || {
            let entry = Entry::new(&service, &key)?;
            entry.set_password(&pass)?;
            Ok::<_, keyring::Error>(())
        })
        .await??;

        Ok(())
    }

    async fn get_password(&self, connection_id: &str) -> Result<String, CredentialError> {
        let service = self.service_name.clone();
        let key = self.get_key(connection_id);

        tokio::task::spawn_blocking(move || Entry::new(&service, &key)?.get_password())
            .await?
            .map_err(|e| Self::not_found(connection_id, e))
    }

    async fn delete_password(&self, connection_id: &str) -> Result<(), CredentialError> {
        let service = self.service_name.clone();
        let key = self.get_key(connection_id);

        tokio::task::spawn_blocking(move || Entry::new(&service, &key)?.delete_credential())
            .await?
            .map_err(|e| Self::not_found(connection_id, e))
    }
}
