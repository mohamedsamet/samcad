//! File dialog adapter using `zenity` binary.
//!
//! This provides native file dialogs on Linux without heavy library dependencies.

use std::path::PathBuf;
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

/// Opens a native save dialog for `.samcad` files.
pub fn save_dialog(default_name: &str) -> Option<PathBuf> {
    let output = Command::new("zenity")
        .arg("--file-selection")
        .arg("--save")
        .arg("--confirm-overwrite")
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

/// Opens a native open dialog for `.samcad` files.
pub fn open_dialog() -> Option<PathBuf> {
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
