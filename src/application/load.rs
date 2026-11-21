//! Load use case: reads a plain JSON `.samcad` file into document and camera.

use std::path::PathBuf;

use crate::core::document::Document;
use crate::core::format::SamcadFile;
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
    #[error(
        "this file uses the old encrypted format; open and re-save it with a previous SamCAD version"
    )]
    LegacyEncryptedFormat,
}

/// Returns true when the content looks like the base64 blob written by the
/// former encrypted format rather than a JSON document.
fn looks_like_legacy_encrypted(content: &str) -> bool {
    let trimmed = content.trim();
    !trimmed.is_empty()
        && !trimmed.starts_with('{')
        && trimmed
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '/' | '=' | '\n' | '\r'))
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
pub struct LoadController<FS: FileSystem> {
    fs: FS,
}

impl<FS: FileSystem> LoadController<FS> {
    pub fn new(fs: FS) -> Self {
        Self { fs }
    }
}

impl<FS: FileSystem> LoadUseCase for LoadController<FS> {
    fn load(&self, path: PathBuf) -> Result<LoadResult, LoadError> {
        let content = self.fs.read(&path)?;

        if looks_like_legacy_encrypted(&content) {
            return Err(LoadError::LegacyEncryptedFormat);
        }

        let file: SamcadFile = serde_json::from_str(&content)?;

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

    const VALID_JSON: &str = r#"{
        "samcad_version": 1,
        "saved_at": "2026-11-20T15:30:00Z",
        "machine": {"hostname": "test", "ip": "127.0.0.1"},
        "document": {"version": 1, "points": [{"x": 1.0, "y": 2.0}, {"x": 3.0, "y": 4.0}]},
        "camera": {"origin_x": 100.0, "origin_y": -50.0, "zoom": 2.5}
    }"#;

    fn json_fixture() -> MemFS {
        MemFS::new_with_file(PathBuf::from("test.samcad"), VALID_JSON.to_string())
    }

    #[test]
    fn load_restores_document_and_camera() {
        let controller = LoadController::new(json_fixture());

        let result = controller.load(PathBuf::from("test.samcad")).unwrap();

        assert_eq!(result.document.points().len(), 2);
        assert_eq!(result.document.points()[0], Point { x: 1.0, y: 2.0 });
        assert_eq!(result.document.points()[1], Point { x: 3.0, y: 4.0 });
        assert_eq!(result.camera.origin().x, 100.0);
        assert_eq!(result.camera.origin().y, -50.0);
        assert_eq!(result.camera.zoom(), 2.5);
    }

    #[test]
    fn load_plain_json_is_portable_across_controllers() {
        let first = LoadController::new(json_fixture());
        let second = LoadController::new(json_fixture());

        let first_result = first.load(PathBuf::from("test.samcad")).unwrap();
        let second_result = second.load(PathBuf::from("test.samcad")).unwrap();

        assert_eq!(
            first_result.document.points(),
            second_result.document.points()
        );
    }

    #[test]
    fn load_missing_file_returns_io_error() {
        let fs = MemFS::new_with_file(PathBuf::from("other.samcad"), "x".to_string());
        let controller = LoadController::new(fs);

        let result = controller.load(PathBuf::from("missing.samcad"));
        assert!(matches!(result, Err(LoadError::Io(_))));
    }

    #[test]
    fn load_legacy_encrypted_data_returns_legacy_format_error() {
        let fs = MemFS::new_with_file(PathBuf::from("test.samcad"), "YWJjZAabc123+/==".to_string());
        let controller = LoadController::new(fs);

        let result = controller.load(PathBuf::from("test.samcad"));
        assert!(matches!(result, Err(LoadError::LegacyEncryptedFormat)));
    }

    #[test]
    fn load_invalid_json_returns_deserialize_error() {
        let fs = MemFS::new_with_file(PathBuf::from("test.samcad"), "not json".to_string());
        let controller = LoadController::new(fs);

        let result = controller.load(PathBuf::from("test.samcad"));
        assert!(matches!(result, Err(LoadError::Deserialize(_))));
    }

    #[test]
    fn load_rejects_unsupported_version() {
        let json = VALID_JSON.replace("\"samcad_version\": 1", "\"samcad_version\": 999");
        let fs = MemFS::new_with_file(PathBuf::from("test.samcad"), json);
        let controller = LoadController::new(fs);

        let err = controller.load(PathBuf::from("test.samcad")).unwrap_err();
        assert!(matches!(err, LoadError::UnsupportedVersion(999)));
    }

    #[test]
    fn open_returns_none_without_dialog() {
        let fs = MemFS::new_with_file(PathBuf::from("test.samcad"), "x".to_string());
        let controller = LoadController::new(fs);

        assert!(controller.open().unwrap().is_none());
    }
}
