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
                    ui.label(&model.title);
                });

                columns[2].with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    for button in &model.window_buttons {
                        paint_window_button(ui, *button, window);
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
