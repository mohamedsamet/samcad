//! Port for secure key storage.
//!
//! The core layer defines the abstraction; concrete implementations
//! (OS keyring, in-memory for tests) live in adapters / tests.

use thiserror::Error;

/// Errors that can occur when accessing the key store.
#[derive(Debug, Error)]
pub enum KeyStoreError {
    #[error("key store unavailable: {0}")]
    Unavailable(String),
    #[error("failed to store key: {0}")]
    Store(String),
    #[error("failed to load key: {0}")]
    Load(String),
    #[error("key not found")]
    NotFound,
}

/// A secure store for the application's encryption key.
///
/// Implementations must keep the key confidential and return the exact
/// bytes that were previously stored.
pub trait KeyStore {
    /// Loads the stored key, or `None` if no key has been stored yet.
    fn load_key(&self) -> Result<Option<Vec<u8>>, KeyStoreError>;

    /// Stores the key bytes, replacing any previous key.
    fn store_key(&self, key: &[u8]) -> Result<(), KeyStoreError>;
}
