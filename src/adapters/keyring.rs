//! OS keyring implementation of the [`KeyStore`] port.
//!
//! Uses the platform-native credential store:
//! - Linux: GNOME Keyring / Secret Service (via D-Bus)
//! - macOS: Keychain
//! - Windows: Credential Manager
//!
//! The encryption key is stored as a binary secret under a fixed
//! service/account pair, so it survives application restarts without
//! ever being exposed in source code, config files, or user prompts.

use keyring::v1::Entry;

use crate::core::keystore::{KeyStore, KeyStoreError};

const SERVICE: &str = "com.samcad.app";
const ACCOUNT: &str = "file-encryption-key";

/// Key store backed by the operating system's credential manager.
pub struct OsKeyStore;

impl OsKeyStore {
    pub fn new() -> Self {
        Self
    }

    fn entry(&self) -> Result<Entry, KeyStoreError> {
        Entry::new(SERVICE, ACCOUNT).map_err(|e| KeyStoreError::Unavailable(e.to_string()))
    }
}

impl Default for OsKeyStore {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyStore for OsKeyStore {
    fn load_key(&self) -> Result<Option<Vec<u8>>, KeyStoreError> {
        let entry = self.entry()?;
        match entry.get_secret() {
            Ok(secret) => Ok(Some(secret)),
            Err(keyring::v1::Error::NoEntry) => Ok(None),
            Err(e) => Err(KeyStoreError::Load(e.to_string())),
        }
    }

    fn store_key(&self, key: &[u8]) -> Result<(), KeyStoreError> {
        let entry = self.entry()?;
        entry
            .set_secret(key)
            .map_err(|e| KeyStoreError::Store(e.to_string()))
    }
}
