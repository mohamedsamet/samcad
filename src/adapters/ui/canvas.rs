use eframe::egui;

use crate::core::document::Point;
use crate::core::ports::{DrawingUseCase, ToolUseCase};
use crate::presentation::canvas::{self, CanvasAction};

use super::cursor;
use super::theme::Theme;

/// Renders the central drawing canvas and the CAD cursor.
///
/// Presentation decides what a click means (`canvas::click_action`) and which
/// points to paint (`canvas::view_model`); this module only translates pointer
/// positions and calls the drawing use case.
pub fn show(
    ui: &mut egui::Ui,
    tools: &dyn ToolUseCase,
    drawing: &dyn DrawingUseCase,
    theme: &Theme,
) {
    egui::CentralPanel::default().show(ui, |ui| {
        let rect = ui.max_rect();
        let response = ui.allocate_rect(rect, egui::Sense::click());

        if ui.ui_contains_pointer() {
            ui.ctx().set_cursor_image(Some(cursor::crosshair().clone()));
            // Fallback for integrations that cannot display a bitmap.
            ui.ctx().set_cursor_icon(egui::CursorIcon::Crosshair);
        }

        if response.clicked()
            && let Some(pos) = response.interact_pointer_pos()
        {
            // Coordinates are canvas-local until a viewport transform exists.
            let local = pos - rect.min;
            if let Some(action) = canvas::click_action(tools.selected(), local.x, local.y) {
                match action {
                    CanvasAction::AddPoint { x, y } => drawing.add_point(Point { x, y }),
                }
            }
        }

        paint_points(ui, rect, drawing, theme);
    });
}

fn paint_points(ui: &egui::Ui, rect: egui::Rect, drawing: &dyn DrawingUseCase, theme: &Theme) {
    let points = drawing.points();
    let painter = ui.painter();
    for point in canvas::view_model(&points) {
        painter.circle_filled(
            rect.min + egui::vec2(point.x, point.y),
            theme.point_radius,
            theme.point_color,
        );
    }
}
