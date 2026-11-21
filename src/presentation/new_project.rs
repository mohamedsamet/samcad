//! Presentation logic for the "New Project" confirmation dialog.
//!
//! Choosing New Project with a non-empty drawing asks whether the current work
//! should be saved. Saving uses the normal Save flow, then starts a fresh
//! project; "Don't Save" starts a fresh project immediately.

use super::dialog::{DialogButton, DialogModel};

/// The choice a user makes in the new-project dialog.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NewProjectChoice {
    /// Save the current work, then start a new project.
    Save,
    /// Start a new project without saving the current work.
    Discard,
}

/// Build the dialog view model. All product decisions live here; the generic
/// dialog adapter only maps these values onto widgets.
pub const fn dialog_model() -> DialogModel<NewProjectChoice> {
    DialogModel {
        title: "New Project",
        message: "Do you want to save the current work before starting a new project?",
        buttons: &[
            DialogButton {
                label: "Save",
                action: NewProjectChoice::Save,
            },
            DialogButton {
                label: "Don't Save",
                action: NewProjectChoice::Discard,
            },
        ],
    }
}
