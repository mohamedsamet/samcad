use eframe::egui;

use crate::presentation::dialog::{DialogModel, DialogState};

use super::theme::Theme;

/// Renders any modal dialog described by a [`DialogModel`].
///
/// Returns the action of the button the user clicked, if any. The dialog closes
/// itself when a button is chosen, when the backdrop is clicked or when Esc is
/// pressed; callers may keep it open by re-opening it after acting.
pub fn show<A: Copy + 'static>(
    ctx: &egui::Context,
    id: egui::Id,
    state: &mut DialogState,
    model: &DialogModel<A>,
    theme: &Theme,
) -> Option<A> {
    if !state.is_open() {
        return None;
    }

    let response = egui::Modal::new(id).show(ctx, |ui| {
        ui.set_width(theme.modal_width);
        ui.heading(model.title);
        ui.label(model.message);
        ui.add_space(theme.modal_spacing);

        let mut chosen = None;
        // Right-to-left layout pins the row to the right edge; reversing the
        // iteration keeps the model's order when read left to right.
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            for button in model.buttons.iter().rev() {
                if ui.button(button.label).clicked() {
                    chosen = Some(button.action);
                }
            }
        });

        chosen
    });

    if response.should_close() {
        state.close();
        return None;
    }

    let chosen = response.inner;
    if chosen.is_some() {
        state.close();
    }
    chosen
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::ui::test_support::Harness;
    use crate::presentation::dialog::DialogButton;

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum TestAction {
        Save,
        Discard,
    }

    const TEST_MODEL: DialogModel<TestAction> = DialogModel {
        title: "Test",
        message: "Test message",
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

    fn theme() -> Theme {
        Theme::default()
    }

    #[test]
    fn dialog_returns_none_when_closed() {
        let mut harness = Harness::new();
        let mut state = DialogState::default();
        let mut result = None;
        harness.frame(vec![], |ui| {
            result = show(
                ui.ctx(),
                egui::Id::unique("test"),
                &mut state,
                &TEST_MODEL,
                &theme(),
            );
        });
        assert_eq!(result, None);
        assert!(!state.is_open());
    }

    #[test]
    fn dialog_opens_when_state_is_open() {
        let mut harness = Harness::new();
        let mut state = DialogState::default();
        state.open();

        let mut result = None;
        harness.frame(vec![], |ui| {
            result = show(
                ui.ctx(),
                egui::Id::unique("test"),
                &mut state,
                &TEST_MODEL,
                &theme(),
            );
        });

        // Dialog should be open after first render (no click yet)
        assert!(state.is_open());
        assert_eq!(result, None);
    }

    #[test]
    fn dialog_closes_on_escape_key() {
        let mut harness = Harness::new();
        let mut state = DialogState::default();
        state.open();

        // First frame opens the dialog
        let mut result = None;
        harness.frame(vec![], |ui| {
            result = show(
                ui.ctx(),
                egui::Id::unique("test"),
                &mut state,
                &TEST_MODEL,
                &theme(),
            );
        });

        assert!(state.is_open());
        assert_eq!(result, None);

        // Press Escape key - modal should close
        harness.frame(
            vec![egui::Event::Key {
                key: egui::Key::Escape,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Default::default(),
            }],
            |ui| {
                result = show(
                    ui.ctx(),
                    egui::Id::unique("test"),
                    &mut state,
                    &TEST_MODEL,
                    &theme(),
                );
            },
        );

        // Dialog should be closed after Escape
        assert!(!state.is_open());
        assert_eq!(result, None);
    }
}
