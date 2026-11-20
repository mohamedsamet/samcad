//! Save use case: serializes the document and camera to an encrypted file.

use std::path::{Path, PathBuf};

use crate::core::crypto::CryptoService;
use crate::core::format::{DocumentData, SamcadFile};
use crate::core::keystore::KeyStore;
use crate::core::machine::collect_machine_info;
use crate::core::ports::DrawingUseCase;
use crate::presentation::canvas::Camera;
use time::OffsetDateTime;

/// Driving port for saving a drawing.
pub trait SaveUseCase {
    fn save(&self, drawing: &dyn DrawingUseCase, camera: &Camera) -> Result<(), SaveError>;
    fn save_as(
        &self,
        path: PathBuf,
        drawing: &dyn DrawingUseCase,
        camera: &Camera,
    ) -> Result<(), SaveError>;
    fn current_path(&self) -> Option<PathBuf>;
}

/// Errors that can occur during save.
#[derive(Debug, thiserror::Error)]
pub enum SaveError {
    #[error("no file path selected")]
    NoPath,
    #[error("serialization failed: {0}")]
    Serialize(#[from] serde_json::Error),
    #[error("file write failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("encryption failed: {0}")]
    Encryption(#[from] crate::core::crypto::CryptoError),
}

/// Driven port: abstract file system for writing.
pub trait FileSystem {
    fn write(&self, path: &Path, content: &str) -> std::io::Result<()>;
}

/// Standard filesystem implementation.
pub struct StdFileSystem;

impl FileSystem for StdFileSystem {
    fn write(&self, path: &Path, content: &str) -> std::io::Result<()> {
        std::fs::write(path, content)
    }
}

/// Controller that implements the save use case.
pub struct SaveController<FS: FileSystem, K: KeyStore> {
    fs: FS,
    crypto: CryptoService<K>,
    current_path: std::cell::RefCell<Option<PathBuf>>,
}

impl<FS: FileSystem, K: KeyStore> SaveController<FS, K> {
    pub fn new(fs: FS, key_store: K) -> Self {
        Self {
            fs,
            crypto: CryptoService::new(key_store),
            current_path: std::cell::RefCell::new(None),
        }
    }

    fn build_samcad_file(
        &self,
        drawing: &dyn DrawingUseCase,
        camera: &Camera,
    ) -> Result<SamcadFile, SaveError> {
        let mut document_data = DocumentData {
            version: DocumentData::CURRENT_VERSION,
            points: Vec::new(),
        };
        drawing.for_each_point(&mut |p| {
            document_data.points.push((*p).into());
        });

        let camera_data = camera.to_data();
        let machine = collect_machine_info();

        let saved_at = OffsetDateTime::now_utc()
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap_or_else(|_| "unknown".to_string());

        Ok(SamcadFile {
            samcad_version: SamcadFile::CURRENT_VERSION,
            saved_at,
            machine,
            document: document_data,
            camera: camera_data,
        })
    }
}

impl<FS: FileSystem, K: KeyStore> SaveUseCase for SaveController<FS, K> {
    fn save(&self, drawing: &dyn DrawingUseCase, camera: &Camera) -> Result<(), SaveError> {
        let path = self
            .current_path
            .borrow()
            .clone()
            .ok_or(SaveError::NoPath)?;
        self.save_as(path, drawing, camera)
    }

    fn save_as(
        &self,
        path: PathBuf,
        drawing: &dyn DrawingUseCase,
        camera: &Camera,
    ) -> Result<(), SaveError> {
        let file = self.build_samcad_file(drawing, camera)?;
        let json = serde_json::to_string_pretty(&file)?;
        let encrypted = self.crypto.encrypt(&json)?;
        self.fs.write(&path, &encrypted)?;

        *self.current_path.borrow_mut() = Some(path);
        Ok(())
    }

    fn current_path(&self) -> Option<PathBuf> {
        self.current_path.borrow().clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::drawing::DrawingController;
    use crate::core::document::Point;
    use crate::core::keystore::KeyStoreError;
    use base64::Engine as _;
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::rc::Rc;

    type Files = Rc<RefCell<HashMap<PathBuf, String>>>;
    type TestController = SaveController<MemFS, MemKeyStore>;
    type ControllerWithFiles = (TestController, Files);

    struct MemFS {
        files: Files,
    }

    impl MemFS {
        fn new() -> Self {
            Self {
                files: Rc::new(RefCell::new(std::collections::HashMap::new())),
            }
        }
    }

    impl FileSystem for MemFS {
        fn write(&self, path: &Path, content: &str) -> std::io::Result<()> {
            self.files
                .borrow_mut()
                .insert(path.to_path_buf(), content.to_string());
            Ok(())
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

    fn new_controller() -> ControllerWithFiles {
        let fs = MemFS::new();
        let files_ref = fs.files.clone();
        (SaveController::new(fs, MemKeyStore::new()), files_ref)
    }

    #[test]
    fn save_writes_encrypted_content() {
        let drawing = DrawingController::new();
        drawing.add_point(Point { x: 1.0, y: 2.0 });
        drawing.add_point(Point { x: 3.0, y: 4.0 });
        let camera = Camera::default();

        let (controller, files) = new_controller();
        let path = PathBuf::from("test.samcad");
        controller.save_as(path.clone(), &drawing, &camera).unwrap();

        let content = files.borrow().get(&path).unwrap().clone();
        // Encrypted file is base64, not plain JSON
        assert!(!content.contains("samcad_version"));
        base64::engine::general_purpose::STANDARD
            .decode(&content)
            .unwrap();
    }

    #[test]
    fn save_updates_current_path() {
        let drawing = DrawingController::new();
        let camera = Camera::default();
        let (controller, _files) = new_controller();

        let path = PathBuf::from("test.samcad");
        controller.save_as(path.clone(), &drawing, &camera).unwrap();

        assert_eq!(controller.current_path(), Some(path));
    }

    #[test]
    fn save_without_path_returns_error() {
        let drawing = DrawingController::new();
        let camera = Camera::default();
        let (controller, _files) = new_controller();

        let result = controller.save(&drawing, &camera);
        assert!(matches!(result, Err(SaveError::NoPath)));
    }

    struct FailingFS;

    impl FileSystem for FailingFS {
        fn write(&self, _path: &Path, _content: &str) -> std::io::Result<()> {
            Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "test failure",
            ))
        }
    }

    #[test]
    fn save_with_failing_filesystem_returns_io_error() {
        let drawing = DrawingController::new();
        let camera = Camera::default();
        let controller = SaveController::new(FailingFS, MemKeyStore::new());

        let result = controller.save_as(PathBuf::from("test.samcad"), &drawing, &camera);
        assert!(matches!(result, Err(SaveError::Io(_))));
    }

    #[test]
    fn save_empty_drawing_succeeds() {
        let drawing = DrawingController::new();
        let camera = Camera::default();
        let (controller, files) = new_controller();

        let path = PathBuf::from("empty.samcad");
        controller.save_as(path.clone(), &drawing, &camera).unwrap();

        assert!(files.borrow().contains_key(&path));
    }

    #[test]
    fn save_to_existing_path_updates_content() {
        let drawing = DrawingController::new();
        drawing.add_point(Point { x: 1.0, y: 2.0 });
        let camera = Camera::default();
        let (controller, files) = new_controller();
        let path = PathBuf::from("test.samcad");

        controller.save_as(path.clone(), &drawing, &camera).unwrap();
        let first = files.borrow().get(&path).unwrap().clone();

        drawing.add_point(Point { x: 3.0, y: 4.0 });
        controller.save(&drawing, &camera).unwrap();
        let second = files.borrow().get(&path).unwrap().clone();

        assert_ne!(first, second);
    }
}
