//! Presentation logic for the "Open File" confirmation dialog.
//!
//! Choosing Open with unsaved changes asks whether the current work should
//! be saved. Saving uses the normal Save flow, then loads the selected file;
//! "Don't Save" loads the selected file immediately.

use super::dialog::{DialogButton, DialogModel};

/// The choice a user makes in the open-project dialog.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OpenProjectChoice {
    /// Save the current work, then load the selected file.
    Save,
    /// Load the selected file without saving the current work.
    Discard,
    /// Cancel the open operation entirely.
    Cancel,
}

/// Build the dialog view model. All product decisions live here; the generic
/// dialog adapter only maps these values onto widgets.
pub const fn dialog_model() -> DialogModel<OpenProjectChoice> {
    DialogModel {
        title: "Open File",
        message: "Do you want to save the current work before opening another file?",
        buttons: &[
            DialogButton {
                label: "Save",
                action: OpenProjectChoice::Save,
            },
            DialogButton {
                label: "Don't Save",
                action: OpenProjectChoice::Discard,
            },
            DialogButton {
                label: "Cancel",
                action: OpenProjectChoice::Cancel,
            },
        ],
    }
}
