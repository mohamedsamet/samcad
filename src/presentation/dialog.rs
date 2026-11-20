//! Generic presentation logic for modal dialogs.
//!
//! A dialog is described by a title, a message and a list of buttons, each
//! carrying the action it represents. Any verification flow (new project,
//! discard changes, ...) can build a [`DialogModel`] and reuse the same
//! adapter.

/// A button offered by a dialog, paired with the action it represents.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DialogButton<A: 'static> {
    pub label: &'static str,
    pub action: A,
}

/// The cooked contents of a modal dialog.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DialogModel<A: 'static> {
    pub title: &'static str,
    pub message: &'static str,
    pub buttons: &'static [DialogButton<A>],
}

/// Open/closed state of a modal dialog.
///
/// Callers keep one per dialog and open it when a confirmation is needed; the
/// adapter reads it to decide whether to paint.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DialogState {
    open: bool,
}

impl DialogState {
    /// Opens the dialog.
    pub fn open(&mut self) {
        self.open = true;
    }

    /// Closes the dialog without acting.
    pub fn close(&mut self) {
        self.open = false;
    }

    /// Whether the dialog is currently shown.
    pub fn is_open(&self) -> bool {
        self.open
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum TestAction {
        Save,
        Discard,
        Cancel,
    }

    #[test]
    fn dialog_state_defaults_to_closed() {
        let state = DialogState::default();
        assert!(!state.is_open());
    }

    #[test]
    fn dialog_state_open_close() {
        let mut state = DialogState::default();
        assert!(!state.is_open());

        state.open();
        assert!(state.is_open());

        state.close();
        assert!(!state.is_open());
    }

    #[test]
    fn dialog_model_holds_buttons_with_actions() {
        let model = DialogModel {
            title: "Test Dialog",
            message: "Choose an option",
            buttons: &[
                DialogButton {
                    label: "Save",
                    action: TestAction::Save,
                },
                DialogButton {
                    label: "Discard",
                    action: TestAction::Discard,
                },
            ],
        };

        assert_eq!(model.title, "Test Dialog");
        assert_eq!(model.message, "Choose an option");
        assert_eq!(model.buttons.len(), 2);
        assert_eq!(model.buttons[0].label, "Save");
        assert_eq!(model.buttons[0].action, TestAction::Save);
        assert_eq!(model.buttons[1].label, "Discard");
        assert_eq!(model.buttons[1].action, TestAction::Discard);
    }

    #[test]
    fn dialog_button_clone_copy() {
        let btn = DialogButton {
            label: "Test",
            action: TestAction::Cancel,
        };

        // These should compile if Clone/Copy are properly derived
        let _copy = btn;
        let _clone = btn; // Copy trait means simple assignment is enough

        assert_eq!(_copy.action, TestAction::Cancel);
        assert_eq!(_clone.action, TestAction::Cancel);
    }
}
