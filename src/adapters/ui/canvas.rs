use eframe::egui;

use super::cursor;

/// Renders the central drawing canvas (currently empty) and switches to the
/// CAD crosshair while the pointer is over it. Other regions keep their own
/// cursor because the custom image is only set here.
pub fn show(ui: &mut egui::Ui) {
    egui::CentralPanel::default().show(ui, |ui| {
        if ui.ui_contains_pointer() {
            ui.ctx().set_cursor_image(Some(cursor::crosshair().clone()));
            // Fallback for integrations that cannot display a bitmap.
            ui.ctx().set_cursor_icon(egui::CursorIcon::Crosshair);
        }
    });
}
