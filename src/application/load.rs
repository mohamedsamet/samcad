//! Load use case: decrypts a `.samcad` file into document and camera.

use std::path::PathBuf;

use crate::core::crypto::CryptoService;
use crate::core::document::Document;
use crate::core::format::SamcadFile;
use crate::core::keystore::KeyStore;
use crate::presentation::canvas::Camera;

/// Driving port for loading a drawing.
pub trait LoadUseCase {
    /// Load from a specific path.
    fn load(&self, path: PathBuf) -> Result<LoadResult, LoadError>;

    /// Open a file dialog and load the selected file.
    fn open(&self) -> Result<Option<LoadResult>, LoadError>;
}

/// Result of a successful load operation.
#[derive(Debug, Clone)]
pub struct LoadResult {
    pub document: Document,
    pub camera: Camera,
}

/// Errors that can occur during load.
#[derive(Debug, thiserror::Error)]
pub enum LoadError {
    #[error("file read failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("deserialization failed: {0}")]
    Deserialize(#[from] serde_json::Error),
    #[error("unsupported file version: {0} (current is {current})", current = SamcadFile::CURRENT_VERSION)]
    UnsupportedVersion(u32),
    #[error("decryption failed: {0}")]
    Decryption(#[from] crate::core::crypto::CryptoError),
}

/// Driven port: abstract file system for reading.
pub trait FileSystem {
    fn read(&self, path: &std::path::Path) -> std::io::Result<String>;
}

/// Standard filesystem implementation.
pub struct StdFileSystem;

impl FileSystem for StdFileSystem {
    fn read(&self, path: &std::path::Path) -> std::io::Result<String> {
        std::fs::read_to_string(path)
    }
}

/// Controller that implements the load use case.
pub struct LoadController<FS: FileSystem, K: KeyStore> {
    fs: FS,
    crypto: CryptoService<K>,
}

impl<FS: FileSystem, K: KeyStore> LoadController<FS, K> {
    pub fn new(fs: FS, key_store: K) -> Self {
        Self {
            fs,
            crypto: CryptoService::new(key_store),
        }
    }
}

impl<FS: FileSystem, K: KeyStore> LoadUseCase for LoadController<FS, K> {
    fn load(&self, path: PathBuf) -> Result<LoadResult, LoadError> {
        let content = self.fs.read(&path)?;
        let json = self.crypto.decrypt(&content)?;

        let file: SamcadFile = serde_json::from_str(&json)?;

        if file.samcad_version != SamcadFile::CURRENT_VERSION {
            return Err(LoadError::UnsupportedVersion(file.samcad_version));
        }

        let document = Document::from_data(file.document);
        let camera = Camera::from_data(file.camera);

        Ok(LoadResult { document, camera })
    }

    fn open(&self) -> Result<Option<LoadResult>, LoadError> {
        // The UI adapter handles the dialog and passes the path to `load()`.
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::document::Point;
    use crate::core::keystore::KeyStoreError;
    use std::cell::RefCell;
    use std::rc::Rc;

    struct MemFS {
        files: Rc<RefCell<std::collections::HashMap<PathBuf, String>>>,
    }

    impl MemFS {
        fn new_with_file(path: PathBuf, content: String) -> Self {
            let mut map = std::collections::HashMap::new();
            map.insert(path, content);
            Self {
                files: Rc::new(RefCell::new(map)),
            }
        }
    }

    impl FileSystem for MemFS {
        fn read(&self, path: &std::path::Path) -> std::io::Result<String> {
            self.files
                .borrow()
                .get(path)
                .cloned()
                .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "file not found"))
        }
    }

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

        fn with_key(key: Vec<u8>) -> Self {
            Self {
                key: Rc::new(RefCell::new(Some(key))),
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

    const VALID_JSON: &str = r#"{
        "samcad_version": 1,
        "saved_at": "2026-11-20T15:30:00Z",
        "machine": {"hostname": "test", "ip": "127.0.0.1"},
        "document": {"version": 1, "points": [{"x": 1.0, "y": 2.0}, {"x": 3.0, "y": 4.0}]},
        "camera": {"origin_x": 100.0, "origin_y": -50.0, "zoom": 2.5}
    }"#;

    fn encrypted_fixture() -> (MemFS, MemKeyStore) {
        let store = MemKeyStore::new();
        let crypto = CryptoService::new(store.clone());
        let encrypted = crypto.encrypt(VALID_JSON).unwrap();
        (
            MemFS::new_with_file(PathBuf::from("test.samcad"), encrypted),
            store,
        )
    }

    #[test]
    fn load_restores_document_and_camera() {
        let (fs, store) = encrypted_fixture();
        let controller = LoadController::new(fs, store);

        let result = controller.load(PathBuf::from("test.samcad")).unwrap();

        assert_eq!(result.document.points().len(), 2);
        assert_eq!(result.document.points()[0], Point { x: 1.0, y: 2.0 });
        assert_eq!(result.document.points()[1], Point { x: 3.0, y: 4.0 });
        assert_eq!(result.camera.origin().x, 100.0);
        assert_eq!(result.camera.origin().y, -50.0);
        assert_eq!(result.camera.zoom(), 2.5);
    }

    #[test]
    fn load_with_wrong_key_fails() {
        let (fs, _correct_store) = encrypted_fixture();
        let wrong_store = MemKeyStore::with_key(vec![9u8; 32]);
        let controller = LoadController::new(fs, wrong_store);

        let result = controller.load(PathBuf::from("test.samcad"));
        assert!(matches!(result, Err(LoadError::Decryption(_))));
    }

    #[test]
    fn load_missing_file_returns_io_error() {
        let fs = MemFS::new_with_file(PathBuf::from("other.samcad"), "x".to_string());
        let controller = LoadController::new(fs, MemKeyStore::new());

        let result = controller.load(PathBuf::from("missing.samcad"));
        assert!(matches!(result, Err(LoadError::Io(_))));
    }

    #[test]
    fn load_corrupted_data_returns_decryption_error() {
        let fs = MemFS::new_with_file(PathBuf::from("test.samcad"), "!!!not_base64!!!".to_string());
        let controller = LoadController::new(fs, MemKeyStore::new());

        let result = controller.load(PathBuf::from("test.samcad"));
        assert!(matches!(result, Err(LoadError::Decryption(_))));
    }

    #[test]
    fn load_encrypted_invalid_json_returns_deserialize_error() {
        let store = MemKeyStore::new();
        let crypto = CryptoService::new(store.clone());
        let encrypted = crypto.encrypt("not json").unwrap();
        let fs = MemFS::new_with_file(PathBuf::from("test.samcad"), encrypted);
        let controller = LoadController::new(fs, store);

        let result = controller.load(PathBuf::from("test.samcad"));
        assert!(matches!(result, Err(LoadError::Deserialize(_))));
    }

    #[test]
    fn load_rejects_unsupported_version() {
        let json = VALID_JSON.replace("\"samcad_version\": 1", "\"samcad_version\": 999");
        let store = MemKeyStore::new();
        let crypto = CryptoService::new(store.clone());
        let encrypted = crypto.encrypt(&json).unwrap();
        let fs = MemFS::new_with_file(PathBuf::from("test.samcad"), encrypted);
        let controller = LoadController::new(fs, store);

        let err = controller.load(PathBuf::from("test.samcad")).unwrap_err();
        assert!(matches!(err, LoadError::UnsupportedVersion(999)));
    }

    #[test]
    fn open_returns_none_without_dialog() {
        let fs = MemFS::new_with_file(PathBuf::from("test.samcad"), "x".to_string());
        let controller = LoadController::new(fs, MemKeyStore::new());

        assert!(controller.open().unwrap().is_none());
    }
}
