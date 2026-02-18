use keyring::Entry;

use crate::connector::commands::error::CredentialError;

pub struct CredentialService {
    service_name: String,
}

impl CredentialService {
    pub fn new(service_name: impl Into<String>) -> Self {
        Self {
            service_name: service_name.into(),
        }
    }

    pub async fn set_password(
        &self,
        connection_id: &str,
        password: &str,
    ) -> Result<(), CredentialError> {
        let service = self.service_name.clone();
        let key = self.get_key(connection_id);
        let pass = password.to_string();

        tokio::task::spawn_blocking(move || {
            let entry = Entry::new(&service, &key)?;
            entry.set_password(&pass)?;
            Ok::<_, keyring::Error>(())
        })
        .await??;

        Ok(())
    }

    pub async fn get_password(
        &self,
        connection_id: &str,
    ) -> Result<String, CredentialError> {
        let service = self.service_name.clone();
        let key = self.get_key(connection_id);

        let password = tokio::task::spawn_blocking(move || {
            let entry = Entry::new(&service, &key)?;
            entry.get_password()
        })
        .await??;

        Ok(password)
    }

    pub async fn delete_password(
        &self,
        connection_id: &str,
    ) -> Result<(), CredentialError> {
        let service = self.service_name.clone();
        let key = self.get_key(connection_id);

        tokio::task::spawn_blocking(move || {
            let entry = Entry::new(&service, &key)?;
            entry.delete_credential()
        })
        .await??;

        Ok(())
    }

    fn get_key(&self, connection_id: &str) -> String {
        format!("connection:{}:basic_password", connection_id)
    }
}
