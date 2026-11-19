use eframe::egui;

use crate::core::ports::WindowUseCase;

/// Renders the custom title bar. Button clicks call the driving port; they
/// never issue viewport commands directly.
pub fn show(ui: &mut egui::Ui, window: &dyn WindowUseCase) {
    egui::Panel::top("title_bar")
        .exact_size(32.0)
        .show(ui, |ui| {
            // Three equal columns keep the middle one centered in the window.
            ui.columns(3, |columns| {
                columns[1].vertical_centered(|ui| {
                    ui.label(format!("SamCad v{}", env!("CARGO_PKG_VERSION")));
                });

                columns[2].with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("×").on_hover_text("Close").clicked() {
                        window.close();
                    }
                    if ui.button("−").on_hover_text("Minimize").clicked() {
                        window.minimize();
                    }
                });
            });
        });
}
