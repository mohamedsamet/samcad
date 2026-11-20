use eframe::egui;

use crate::core::ports::WindowUseCase;
use crate::presentation::component::WindowButton;
use crate::presentation::main_bar;

use super::theme::Theme;

/// Renders the permanent main bar: version title in the middle, window
/// controls on the right. Button clicks call the driving port; they never
/// issue viewport commands directly.
pub fn show(ui: &mut egui::Ui, window: &dyn WindowUseCase, theme: &Theme) {
    let model = main_bar::view_model();

    egui::Panel::top("main_bar")
        .exact_size(theme.main_bar_height)
        .show(ui, |ui| {
            // Three equal columns keep the middle one centered in the window.
            ui.columns(3, |columns| {
                columns[1].vertical_centered(|ui| {
                    ui.label(model.title);
                });

                // Right-to-left layout puts the first item at the far right,
                // so iterate in reverse to keep Close at the edge and Minimize
                // to its left (the conventional order).
                columns[2].with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    for &button in model.window_buttons.iter().rev() {
                        paint_window_button(ui, button, window);
                    }
                });
            });
        });
}

fn paint_window_button(ui: &mut egui::Ui, button: WindowButton, window: &dyn WindowUseCase) {
    let (glyph, tooltip) = match button {
        WindowButton::Minimize => ("−", "Minimize"),
        WindowButton::Close => ("×", "Close"),
    };

    if ui.button(glyph).on_hover_text(tooltip).clicked() {
        match button {
            WindowButton::Minimize => window.minimize(),
            WindowButton::Close => window.close(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::adapters::ui::test_support::{Harness, RecordingWindow};

    #[test]
    fn render_keeps_the_bar_free_of_window_commands_until_clicked() {
        let mut harness = Harness::new();
        let window = RecordingWindow::default();

        harness.frame(vec![], |ui| show(ui, &window, &Theme::default()));

        assert!(window.calls.borrow().is_empty());
    }

    #[test]
    fn both_window_buttons_call_their_window_port_method() {
        let mut harness = Harness::new();
        let window = RecordingWindow::default();
        let button_pos = egui::pos2(10.0, 10.0);

        for button in [WindowButton::Minimize, WindowButton::Close] {
            harness.click(button_pos, |ui| {
                paint_window_button(ui, button, &window);
            });
        }

        assert_eq!(*window.calls.borrow(), ["minimize", "close"]);
    }
}
