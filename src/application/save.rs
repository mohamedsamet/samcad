//! Save use case: serializes the document and camera to a plain JSON file.

use std::path::{Path, PathBuf};

use crate::core::format::{DocumentData, SamcadFile};
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
pub struct SaveController<FS: FileSystem> {
    fs: FS,
    current_path: std::cell::RefCell<Option<PathBuf>>,
}

impl<FS: FileSystem> SaveController<FS> {
    pub fn new(fs: FS) -> Self {
        Self {
            fs,
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

impl<FS: FileSystem> SaveUseCase for SaveController<FS> {
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
        self.fs.write(&path, &json)?;

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
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::rc::Rc;

    type Files = Rc<RefCell<HashMap<PathBuf, String>>>;
    type TestController = SaveController<MemFS>;
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

    fn new_controller() -> ControllerWithFiles {
        let fs = MemFS::new();
        let files_ref = fs.files.clone();
        (SaveController::new(fs), files_ref)
    }

    #[test]
    fn save_writes_plain_json_content() {
        let drawing = DrawingController::new();
        drawing.add_point(Point { x: 1.0, y: 2.0 });
        drawing.add_point(Point { x: 3.0, y: 4.0 });
        let camera = Camera::default();

        let (controller, files) = new_controller();
        let path = PathBuf::from("test.samcad");
        controller.save_as(path.clone(), &drawing, &camera).unwrap();

        let content = files.borrow().get(&path).unwrap().clone();
        assert!(content.contains("samcad_version"));
        let parsed: SamcadFile = serde_json::from_str(&content).unwrap();
        assert_eq!(parsed.samcad_version, SamcadFile::CURRENT_VERSION);
        assert_eq!(parsed.document.points.len(), 2);
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
        let controller = SaveController::new(FailingFS);

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
