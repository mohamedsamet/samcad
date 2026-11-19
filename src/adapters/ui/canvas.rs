use eframe::egui;

use crate::core::ports::{DrawingUseCase, ToolUseCase};
use crate::presentation::canvas::{self, Camera, CanvasAction};

use super::cursor;
use super::theme::Theme;

/// Wheel sensitivity: a notch of ~50 points gives roughly a 10% zoom step.
const ZOOM_SENSITIVITY: f32 = 0.002;

/// Renders the central drawing canvas, the infinite-canvas camera and the CAD
/// cursor.
///
/// Presentation decides what a click means and how world coordinates map to the
/// screen; this module only feeds pointer input to the camera and paints the
/// visible points.
pub fn show(
    ui: &mut egui::Ui,
    tools: &dyn ToolUseCase,
    drawing: &dyn DrawingUseCase,
    camera: &mut Camera,
    theme: &Theme,
) {
    egui::CentralPanel::default().show(ui, |ui| {
        let rect = ui.max_rect();
        let response = ui.allocate_rect(rect, egui::Sense::click_and_drag());

        if ui.ui_contains_pointer() {
            ui.ctx().set_cursor_image(Some(cursor::crosshair().clone()));
            // Fallback for integrations that cannot display a bitmap.
            ui.ctx().set_cursor_icon(egui::CursorIcon::Crosshair);
        }

        update_camera(ui, &response, rect, tools, camera);

        if response.clicked()
            && let Some(pos) = response.interact_pointer_pos()
        {
            let local = pos - rect.min;
            let world = camera.screen_to_world(local.x, local.y);
            if let Some(action) = canvas::click_action(tools.selected(), world) {
                match action {
                    CanvasAction::AddPoint(point) => drawing.add_point(point),
                }
            }
        }

        paint_points(ui, rect, drawing, camera, theme);
    });
}

/// Applies wheel zoom (anchored at the cursor) and drag panning.
fn update_camera(
    ui: &egui::Ui,
    response: &egui::Response,
    rect: egui::Rect,
    tools: &dyn ToolUseCase,
    camera: &mut Camera,
) {
    if response.hovered()
        && let Some(pos) = response.hover_pos()
    {
        // Plain wheel zooms; Ctrl+wheel and trackpad pinch arrive as a zoom
        // factor instead. Exactly one of the two is non-neutral per frame.
        let (scroll, pinch) = ui
            .ctx()
            .input(|input| (input.smooth_scroll_delta.y, input.zoom_delta()));
        let factor = pinch * (scroll * ZOOM_SENSITIVITY).exp();
        if factor != 1.0 {
            let local = pos - rect.min;
            camera.zoom_around(local.x, local.y, factor);
        }
    }

    // Middle-drag always pans; primary-drag pans only when no tool is active,
    // so it never competes with click-to-draw.
    let primary_pans = tools.selected().is_none();
    if response.dragged_by(egui::PointerButton::Middle)
        || (primary_pans && response.dragged_by(egui::PointerButton::Primary))
    {
        let delta = response.drag_delta();
        camera.pan_by(delta.x, delta.y);
    }
}

/// Paints only the points that fall inside the canvas. No allocation: the
/// document is visited in place, so frame cost is proportional to the number of
/// stored points and idle frames are not requested.
fn paint_points(
    ui: &egui::Ui,
    rect: egui::Rect,
    drawing: &dyn DrawingUseCase,
    camera: &Camera,
    theme: &Theme,
) {
    let painter = ui.painter();
    let width = rect.width();
    let height = rect.height();
    let margin = theme.point_radius;

    drawing.for_each_point(&mut |point| {
        let screen = camera.world_to_screen(point);
        if screen.x >= -margin
            && screen.x <= width + margin
            && screen.y >= -margin
            && screen.y <= height + margin
        {
            painter.circle_filled(
                rect.min + egui::vec2(screen.x, screen.y),
                theme.point_radius,
                theme.point_color,
            );
        }
    });
}
