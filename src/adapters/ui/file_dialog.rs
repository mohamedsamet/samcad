//! File dialog adapter using `zenity` binary.
//!
//! This provides native file dialogs on Linux without heavy library dependencies.

use std::path::{Path, PathBuf};
use std::process::Command;

const SAMCAD_EXTENSION: &str = "samcad";

/// Ensures the path has the `.samcad` extension.
fn ensure_samcad_extension(path: PathBuf) -> PathBuf {
    if path.extension().and_then(|e| e.to_str()) != Some(SAMCAD_EXTENSION) {
        let mut new_path = path;
        new_path.set_extension(SAMCAD_EXTENSION);
        new_path
    } else {
        path
    }
}

/// Returns a path that does not exist yet, adding a numeric suffix if needed.
fn unique_path(path: PathBuf) -> PathBuf {
    if !path.exists() {
        return path;
    }

    let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("untitled");
    let parent = path.parent().unwrap_or_else(|| Path::new(""));

    for number in 2.. {
        let name = if extension.is_empty() {
            format!("{} ({})", stem, number)
        } else {
            format!("{} ({}).{}", stem, number, extension)
        };
        let candidate = parent.join(name);
        if !candidate.exists() {
            return candidate;
        }
    }

    unreachable!("unique path search always terminates")
}

/// Trait for file dialog operations - allows mocking in tests.
pub trait FileDialog: Send + Sync {
    /// Opens a native save dialog for `.samcad` files.
    fn save_dialog(&self, default_name: &str) -> Option<PathBuf>;

    /// Opens a native save dialog for a brand-new `.samcad` file.
    /// Never returns a path that already exists.
    fn save_new_dialog(&self, default_name: &str) -> Option<PathBuf>;

    /// Opens a native open dialog for `.samcad` files.
    fn open_dialog(&self) -> Option<PathBuf>;
}

/// Real implementation using zenity.
pub struct ZenityFileDialog;

impl FileDialog for ZenityFileDialog {
    fn save_dialog(&self, default_name: &str) -> Option<PathBuf> {
        run_save_dialog(default_name, true)
    }

    fn save_new_dialog(&self, default_name: &str) -> Option<PathBuf> {
        run_save_dialog(default_name, false).map(unique_path)
    }

    fn open_dialog(&self) -> Option<PathBuf> {
        let output = Command::new("zenity")
            .arg("--file-selection")
            .arg("--file-filter=SamCAD files (*.samcad) | *.samcad")
            .arg("--file-filter=All files | *")
            .output()
            .ok()?;

        if output.status.success() {
            let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !path_str.is_empty() {
                return Some(ensure_samcad_extension(PathBuf::from(path_str)));
            }
        }
        None
    }
}

fn run_save_dialog(default_name: &str, confirm_overwrite: bool) -> Option<PathBuf> {
    let mut command = Command::new("zenity");
    command.arg("--file-selection").arg("--save");
    if confirm_overwrite {
        command.arg("--confirm-overwrite");
    }

    let output = command
        .arg(format!(
            "--filename={}",
            ensure_samcad_extension(PathBuf::from(default_name)).display()
        ))
        .arg("--file-filter=SamCAD files (*.samcad) | *.samcad")
        .arg("--file-filter=All files | *")
        .output()
        .ok()?;

    if output.status.success() {
        let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !path_str.is_empty() {
            return Some(ensure_samcad_extension(PathBuf::from(path_str)));
        }
    }
    None
}

/// Mock implementation for testing - returns None to simulate user cancellation.
#[cfg(test)]
pub struct MockFileDialog {
    pub save_dialog_result: Option<PathBuf>,
    pub save_new_dialog_result: Option<PathBuf>,
    pub open_dialog_result: Option<PathBuf>,
}

#[cfg(test)]
impl Default for MockFileDialog {
    fn default() -> Self {
        Self {
            save_dialog_result: None,
            save_new_dialog_result: None,
            open_dialog_result: None,
        }
    }
}

#[cfg(test)]
impl FileDialog for MockFileDialog {
    fn save_dialog(&self, _default_name: &str) -> Option<PathBuf> {
        self.save_dialog_result.clone()
    }

    fn save_new_dialog(&self, _default_name: &str) -> Option<PathBuf> {
        self.save_new_dialog_result.clone()
    }

    fn open_dialog(&self) -> Option<PathBuf> {
        self.open_dialog_result.clone()
    }
}

/// Opens a native save dialog for `.samcad` files.
///
/// Used for the first save of a project; overwriting an existing file is
/// confirmed by the dialog itself.
pub fn save_dialog(default_name: &str) -> Option<PathBuf> {
    ZenityFileDialog.save_dialog(default_name)
}

/// Opens a native save dialog for a brand-new `.samcad` file.
///
/// Never returns a path that already exists: an existing selection is given a
/// numeric suffix instead.
pub fn save_new_dialog(default_name: &str) -> Option<PathBuf> {
    ZenityFileDialog.save_new_dialog(default_name)
}

/// Opens a native open dialog for `.samcad` files.
pub fn open_dialog() -> Option<PathBuf> {
    ZenityFileDialog.open_dialog()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn ensure_extension_adds_samcad() {
        let path = PathBuf::from("drawing");
        let result = ensure_samcad_extension(path);
        assert_eq!(result, PathBuf::from("drawing.samcad"));
    }

    #[test]
    fn ensure_extension_keeps_existing_samcad() {
        let path = PathBuf::from("drawing.samcad");
        let result = ensure_samcad_extension(path.clone());
        assert_eq!(result, path);
    }

    #[test]
    fn ensure_extension_replaces_other_extension() {
        let path = PathBuf::from("drawing.txt");
        let result = ensure_samcad_extension(path);
        assert_eq!(result, PathBuf::from("drawing.samcad"));
    }

    #[test]
    fn ensure_extension_handles_nested_path() {
        let path = PathBuf::from("/home/user/drawings/myfile");
        let result = ensure_samcad_extension(path);
        assert_eq!(result, PathBuf::from("/home/user/drawings/myfile.samcad"));
    }

    #[test]
    fn ensure_extension_handles_hidden_file() {
        let path = PathBuf::from(".hidden");
        let result = ensure_samcad_extension(path);
        assert_eq!(result, PathBuf::from(".hidden.samcad"));
    }

    #[test]
    fn unique_path_returns_same_when_not_exists() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("new_file.samcad");
        let result = unique_path(path.clone());
        assert_eq!(result, path);
    }

    #[test]
    fn unique_path_appends_number_when_exists() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("file.samcad");
        fs::write(&path, b"").unwrap();

        let result = unique_path(path.clone());
        assert_eq!(result, dir.path().join("file (2).samcad"));
    }

    #[test]
    fn unique_path_increments_when_multiple_exist() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("file.samcad");
        fs::write(&path, b"").unwrap();
        fs::write(dir.path().join("file (2).samcad"), b"").unwrap();

        let result = unique_path(path.clone());
        assert_eq!(result, dir.path().join("file (3).samcad"));
    }

    #[test]
    fn unique_path_works_without_extension() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("file");
        fs::write(&path, b"").unwrap();

        let result = unique_path(path.clone());
        assert_eq!(result, dir.path().join("file (2)"));
    }

    #[test]
    fn unique_path_works_with_hidden_file() {
        let dir = tempdir().unwrap();
        let path = dir.path().join(".hidden.samcad");
        fs::write(&path, b"").unwrap();

        let result = unique_path(path.clone());
        assert_eq!(result, dir.path().join(".hidden (2).samcad"));
    }
}
