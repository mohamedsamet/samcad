//! Encryption/decryption service for `.samcad` files.
//!
//! Uses AES-256-GCM authenticated encryption with a 256-bit key stored in the
//! OS keyring (via the [`KeyStore`] port). No password is ever requested from
//! the user or written anywhere in the project.
//!
//! File format: base64 of `[nonce: 12 bytes][ciphertext + tag]`.

use std::path::Path;

use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, KeyInit},
};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use rand::{RngCore, rngs::OsRng};
use thiserror::Error;
use zeroize::Zeroize;

use crate::core::keystore::{KeyStore, KeyStoreError};

const KEY_LEN: usize = 32;
const NONCE_LEN: usize = 12;

/// Errors that can occur during encryption/decryption.
#[derive(Debug, Error)]
pub enum CryptoError {
    #[error("encryption failed: {0}")]
    Encryption(String),
    #[error("decryption failed: {0}")]
    Decryption(String),
    #[error("corrupted or tampered file")]
    CorruptedFile,
    #[error("invalid file format")]
    InvalidFormat,
    #[error("key store error: {0}")]
    KeyStore(#[from] KeyStoreError),
    #[error("stored key has invalid length {0}, expected {expected}", expected = KEY_LEN)]
    InvalidKeyLength(usize),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Service for encrypting and decrypting file contents using a stored key.
pub struct CryptoService<K: KeyStore> {
    key_store: K,
}

impl<K: KeyStore> CryptoService<K> {
    /// Creates a new crypto service backed by the given key store.
    pub fn new(key_store: K) -> Self {
        Self { key_store }
    }

    /// Loads the key from the store, generating and persisting a fresh
    /// 256-bit key on first use. The key is zeroized on drop.
    fn load_or_create_key(&self) -> Result<[u8; KEY_LEN], CryptoError> {
        let maybe_key = self.key_store.load_key()?;

        let key_bytes = match maybe_key {
            Some(bytes) => bytes,
            None => {
                let mut fresh = [0u8; KEY_LEN];
                OsRng.fill_bytes(&mut fresh);
                self.key_store.store_key(&fresh)?;
                fresh.to_vec()
            }
        };

        if key_bytes.len() != KEY_LEN {
            return Err(CryptoError::InvalidKeyLength(key_bytes.len()));
        }

        let mut key = [0u8; KEY_LEN];
        key.copy_from_slice(&key_bytes);
        Ok(key)
    }

    /// Encrypts data with the stored key.
    ///
    /// Returns base64 of `nonce || ciphertext+tag`.
    pub fn encrypt(&self, data: &str) -> Result<String, CryptoError> {
        let mut key = self.load_or_create_key()?;

        let result = (|| {
            let cipher = Aes256Gcm::new((&key).into());

            let mut nonce_bytes = [0u8; NONCE_LEN];
            OsRng.fill_bytes(&mut nonce_bytes);
            let nonce = Nonce::from_slice(&nonce_bytes);

            let ciphertext = cipher
                .encrypt(nonce, data.as_bytes())
                .map_err(|e| CryptoError::Encryption(e.to_string()))?;

            let mut combined = Vec::with_capacity(NONCE_LEN + ciphertext.len());
            combined.extend_from_slice(&nonce_bytes);
            combined.extend_from_slice(&ciphertext);

            Ok(BASE64.encode(combined))
        })();

        key.zeroize();
        result
    }

    /// Decrypts base64-encoded data with the stored key.
    pub fn decrypt(&self, encoded: &str) -> Result<String, CryptoError> {
        let mut key = self.load_or_create_key()?;

        let result = (|| {
            let combined = BASE64
                .decode(encoded)
                .map_err(|e| CryptoError::Decryption(e.to_string()))?;

            if combined.len() < NONCE_LEN {
                return Err(CryptoError::InvalidFormat);
            }

            let nonce = Nonce::from_slice(&combined[..NONCE_LEN]);
            let ciphertext = &combined[NONCE_LEN..];

            let cipher = Aes256Gcm::new((&key).into());
            let plaintext = cipher
                .decrypt(nonce, ciphertext)
                .map_err(|_| CryptoError::CorruptedFile)?;

            String::from_utf8(plaintext).map_err(|e| CryptoError::Decryption(e.to_string()))
        })();

        key.zeroize();
        result
    }

    /// Encrypts and writes data to a file.
    pub fn encrypt_to_file(&self, path: &Path, data: &str) -> Result<(), CryptoError> {
        let encrypted = self.encrypt(data)?;
        std::fs::write(path, encrypted)?;
        Ok(())
    }

    /// Reads and decrypts data from a file.
    pub fn decrypt_from_file(&self, path: &Path) -> Result<String, CryptoError> {
        let encoded = std::fs::read_to_string(path)?;
        self.decrypt(&encoded)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[derive(Clone)]
    struct MemKeyStore {
        key: Rc<RefCell<Option<Vec<u8>>>>,
    }

    impl MemKeyStore {
        fn new() -> Self {
            Self {
                key: Rc::new(RefCell::new(None)),
            }
        }
    }

    impl KeyStore for MemKeyStore {
        fn load_key(&self) -> Result<Option<Vec<u8>>, KeyStoreError> {
            Ok(self.key.borrow().clone())
        }

        fn store_key(&self, key: &[u8]) -> Result<(), KeyStoreError> {
            *self.key.borrow_mut() = Some(key.to_vec());
            Ok(())
        }
    }

    #[test]
    fn encrypt_decrypt_round_trip() {
        let store = MemKeyStore::new();
        let service = CryptoService::new(store);
        let data = r#"{"test": "data", "number": 42}"#;

        let encrypted = service.encrypt(data).unwrap();
        let decrypted = service.decrypt(&encrypted).unwrap();

        assert_eq!(data, decrypted);
    }

    #[test]
    fn first_encrypt_generates_and_stores_key() {
        let store = MemKeyStore::new();
        let service = CryptoService::new(store.clone());

        service.encrypt("data").unwrap();

        assert!(store.key.borrow().is_some());
        assert_eq!(store.key.borrow().as_ref().unwrap().len(), KEY_LEN);
    }

    #[test]
    fn key_is_reused_across_calls() {
        let store = MemKeyStore::new();
        let service = CryptoService::new(store.clone());

        service.encrypt("first").unwrap();
        let key_after_first = store.key.borrow().clone();
        service.encrypt("second").unwrap();
        let key_after_second = store.key.borrow().clone();

        assert_eq!(key_after_first, key_after_second);
    }

    #[test]
    fn different_nonce_produces_different_ciphertext() {
        let store = MemKeyStore::new();
        let service = CryptoService::new(store);

        let enc1 = service.encrypt("same data").unwrap();
        let enc2 = service.encrypt("same data").unwrap();

        assert_ne!(enc1, enc2);
    }

    #[test]
    fn decrypt_with_different_key_fails() {
        let store1 = MemKeyStore::new();
        let store2 = MemKeyStore::new();
        let service1 = CryptoService::new(store1);
        let service2 = CryptoService::new(store2);

        let encrypted = service1.encrypt("secret").unwrap();
        let result = service2.decrypt(&encrypted);

        assert!(matches!(result, Err(CryptoError::CorruptedFile)));
    }

    #[test]
    fn decrypt_with_invalid_base64_returns_error() {
        let store = MemKeyStore::new();
        let service = CryptoService::new(store);

        let result = service.decrypt("not_valid_base64!!!");
        assert!(matches!(result, Err(CryptoError::Decryption(_))));
    }

    #[test]
    fn decrypt_with_too_short_data_returns_invalid_format() {
        let store = MemKeyStore::new();
        let service = CryptoService::new(store);
        let short_data = BASE64.encode([0u8; 5]);

        let result = service.decrypt(&short_data);
        assert!(matches!(result, Err(CryptoError::InvalidFormat)));
    }

    #[test]
    fn invalid_stored_key_length_returns_error() {
        let store = MemKeyStore::new();
        *store.key.borrow_mut() = Some(vec![0u8; 16]); // wrong length
        let service = CryptoService::new(store);

        let result = service.encrypt("data");
        assert!(matches!(result, Err(CryptoError::InvalidKeyLength(16))));
    }

    #[test]
    fn key_store_error_propagates() {
        struct FailingKeyStore;
        impl KeyStore for FailingKeyStore {
            fn load_key(&self) -> Result<Option<Vec<u8>>, KeyStoreError> {
                Err(KeyStoreError::Unavailable("test".to_string()))
            }
            fn store_key(&self, _key: &[u8]) -> Result<(), KeyStoreError> {
                Err(KeyStoreError::Store("test".to_string()))
            }
        }

        let service = CryptoService::new(FailingKeyStore);
        let result = service.encrypt("data");
        assert!(matches!(result, Err(CryptoError::KeyStore(_))));
    }

    #[test]
    fn store_key_failure_propagates() {
        struct FailingStoreKeyStore;
        impl KeyStore for FailingStoreKeyStore {
            fn load_key(&self) -> Result<Option<Vec<u8>>, KeyStoreError> {
                Ok(None)
            }
            fn store_key(&self, _key: &[u8]) -> Result<(), KeyStoreError> {
                Err(KeyStoreError::Store("test".to_string()))
            }
        }

        let service = CryptoService::new(FailingStoreKeyStore);
        let result = service.encrypt("data");
        assert!(matches!(result, Err(CryptoError::KeyStore(_))));
    }

    #[test]
    fn encrypt_to_file_and_decrypt() {
        let store = MemKeyStore::new();
        let service = CryptoService::new(store);
        let data = r#"{"points": [{"x": 1.0, "y": 2.0}]}"#;
        let temp_path = std::env::temp_dir().join("test_keyring_encrypt.samcad");

        service.encrypt_to_file(&temp_path, data).unwrap();
        let decrypted = service.decrypt_from_file(&temp_path).unwrap();

        assert_eq!(data, decrypted);
        std::fs::remove_file(temp_path).ok();
    }

    #[test]
    fn decrypt_from_missing_file_returns_io_error() {
        let store = MemKeyStore::new();
        let service = CryptoService::new(store);
        let path = std::env::temp_dir().join("nonexistent_keyring_test_12345.samcad");

        let result = service.decrypt_from_file(&path);
        assert!(matches!(result, Err(CryptoError::Io(_))));
    }

    #[test]
    fn tampered_file_fails() {
        let store = MemKeyStore::new();
        let service = CryptoService::new(store);
        let encrypted = service.encrypt("secret").unwrap();

        let mut bytes = BASE64.decode(&encrypted).unwrap();
        bytes[NONCE_LEN] ^= 0xFF; // flip a ciphertext bit
        let tampered = BASE64.encode(bytes);

        let result = service.decrypt(&tampered);
        assert!(matches!(result, Err(CryptoError::CorruptedFile)));
    }

    #[test]
    fn empty_data_encrypts_and_decrypts() {
        let store = MemKeyStore::new();
        let service = CryptoService::new(store);

        let encrypted = service.encrypt("").unwrap();
        let decrypted = service.decrypt(&encrypted).unwrap();

        assert_eq!("", decrypted);
    }

    #[test]
    fn unicode_data_encrypts_and_decrypts() {
        let store = MemKeyStore::new();
        let service = CryptoService::new(store);
        let data = "🔐 Hello, 世界! éèê";

        let encrypted = service.encrypt(data).unwrap();
        let decrypted = service.decrypt(&encrypted).unwrap();

        assert_eq!(data, decrypted);
    }
}
