use async_trait::async_trait;

use crate::connector::commands::error::CredentialError;

/// Where connection passwords live: the OS keyring in the app, memory in tests.
#[async_trait]
pub trait CredentialStore: Send + Sync {
    async fn set_password(&self, connection_id: &str, password: &str)
        -> Result<(), CredentialError>;
    /// `CredentialError::NotFound` when nothing is stored for the connection.
    async fn get_password(&self, connection_id: &str) -> Result<String, CredentialError>;
    async fn delete_password(&self, connection_id: &str) -> Result<(), CredentialError>;
}

#[cfg(test)]
pub mod memory {
    use std::collections::HashMap;
    use std::sync::Mutex;

    use async_trait::async_trait;

    use super::CredentialStore;
    use crate::connector::commands::error::CredentialError;

    #[derive(Default)]
    pub struct MemoryCredentialStore {
        passwords: Mutex<HashMap<String, String>>,
        fail: Mutex<bool>,
    }

    impl MemoryCredentialStore {
        pub fn stored(&self, connection_id: &str) -> Option<String> {
            self.passwords.lock().unwrap().get(connection_id).cloned()
        }

        pub fn insert(&self, connection_id: &str, password: &str) {
            self.passwords
                .lock()
                .unwrap()
                .insert(connection_id.to_string(), password.to_string());
        }

        /// Every following call fails like a locked or missing keyring would.
        pub fn fail_from_now_on(&self) {
            *self.fail.lock().unwrap() = true;
        }

        fn check(&self) -> Result<(), CredentialError> {
            if *self.fail.lock().unwrap() {
                return Err(CredentialError::Keyring(keyring::Error::NoStorageAccess(
                    "test store fails".into(),
                )));
            }
            Ok(())
        }
    }

    #[async_trait]
    impl CredentialStore for MemoryCredentialStore {
        async fn set_password(
            &self,
            connection_id: &str,
            password: &str,
        ) -> Result<(), CredentialError> {
            self.check()?;
            self.insert(connection_id, password);
            Ok(())
        }

        async fn get_password(&self, connection_id: &str) -> Result<String, CredentialError> {
            self.check()?;
            self.stored(connection_id)
                .ok_or_else(|| CredentialError::NotFound(connection_id.to_string()))
        }

        async fn delete_password(&self, connection_id: &str) -> Result<(), CredentialError> {
            self.check()?;
            self.passwords
                .lock()
                .unwrap()
                .remove(connection_id)
                .map(|_| ())
                .ok_or_else(|| CredentialError::NotFound(connection_id.to_string()))
        }
    }
}
