use keyring::Entry;
use std::error::Error;

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
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let service = self.service_name.clone();
        let id = connection_id.to_string();
        let pass = password.to_string();
        let key = self.get_key(&id);

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
    ) -> Result<String, Box<dyn Error + Send + Sync>> {
        let service = self.service_name.clone();
        let id = connection_id.to_string();
        let key = self.get_key(&id);

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
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let service = self.service_name.clone();
        let id = connection_id.to_string();
        let key = self.get_key(&id);

        tokio::task::spawn_blocking(move || {
            let entry = Entry::new(&service, &key)?;
            entry.delete_credential()
        })
        .await??;

        Ok(())
    }

    fn get_key(&self, connection_id: &str) -> String {
        format!("connection:{}:password", connection_id)
    }
}
