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

#[cfg(test)]
mod tests {
    use super::*;

    use crate::adapters::ui::test_support::{Harness, button, moved, points};
    use crate::application::drawing::DrawingController;
    use crate::application::tool::ToolController;
    use crate::core::document::Point;
    use crate::core::model::Tool;

    const CURSOR: egui::Pos2 = egui::pos2(600.0, 400.0);
    // egui's default central panel frame has an 8-point inner margin, so the
    // canvas-local origin is offset from the window origin in these tests.
    const CANVAS_ORIGIN: egui::Vec2 = egui::vec2(8.0, 8.0);

    #[test]
    fn point_click_creates_a_world_point_and_paints_visible_ones() {
        let mut harness = Harness::new();
        let tools = ToolController::new();
        tools.select(Tool::Point);
        let drawing = DrawingController::new();
        // Outside the viewport: exercised by culling during painting.
        drawing.add_point(Point {
            x: 9_000.0,
            y: 9_000.0,
        });
        let mut camera = Camera::default();

        harness.click(egui::pos2(300.0, 200.0), |ui| {
            show(ui, &tools, &drawing, &mut camera, &Theme::default());
        });

        assert_eq!(
            points(&drawing),
            [
                Point {
                    x: 9_000.0,
                    y: 9_000.0
                },
                Point {
                    x: 300.0 - CANVAS_ORIGIN.x as f64,
                    y: -(200.0 - CANVAS_ORIGIN.y as f64)
                },
            ]
        );
    }

    #[test]
    fn click_without_a_tool_does_not_change_the_drawing() {
        let mut harness = Harness::new();
        let tools = ToolController::new();
        let drawing = DrawingController::new();
        let mut camera = Camera::default();

        harness.click(CURSOR, |ui| {
            show(ui, &tools, &drawing, &mut camera, &Theme::default());
        });

        assert!(points(&drawing).is_empty());
    }

    #[test]
    fn pinch_zoom_stays_anchored_under_the_cursor() {
        let mut harness = Harness::new();
        let tools = ToolController::new();
        let drawing = DrawingController::new();
        let mut camera = Camera::default();
        let anchor = CURSOR - CANVAS_ORIGIN;
        let before = camera.screen_to_world(anchor.x, anchor.y);
        let mut render = |ui: &mut egui::Ui| {
            show(ui, &tools, &drawing, &mut camera, &Theme::default());
        };

        harness.frame(vec![moved(CURSOR)], &mut render);
        harness.frame(vec![moved(CURSOR), egui::Event::Zoom(2.0)], render);

        let after = camera.screen_to_world(anchor.x, anchor.y);
        assert!((after.x - before.x).abs() < 1e-6);
        assert!((after.y - before.y).abs() < 1e-6);
        assert_eq!(camera.zoom(), 2.0);
    }

    #[test]
    fn middle_drag_pans_even_with_a_tool_selected() {
        let mut harness = Harness::new();
        let tools = ToolController::new();
        tools.select(Tool::Point);
        let drawing = DrawingController::new();
        let mut camera = Camera::default();
        let end = egui::pos2(CURSOR.x + 50.0, CURSOR.y + 30.0);
        let mut render = |ui: &mut egui::Ui| {
            show(ui, &tools, &drawing, &mut camera, &Theme::default());
        };

        harness.frame(vec![moved(CURSOR)], &mut render);
        harness.frame(
            vec![
                moved(CURSOR),
                button(CURSOR, egui::PointerButton::Middle, true),
            ],
            &mut render,
        );
        harness.frame(vec![moved(end)], &mut render);
        harness.frame(
            vec![button(end, egui::PointerButton::Middle, false)],
            render,
        );

        assert_eq!(
            camera.screen_to_world(0.0, 0.0),
            Point { x: -50.0, y: 30.0 }
        );
    }

    #[test]
    fn primary_drag_pans_only_when_no_tool_is_active() {
        let mut harness = Harness::new();
        let tools = ToolController::new();
        let drawing = DrawingController::new();
        let mut camera = Camera::default();
        let end = egui::pos2(CURSOR.x - 20.0, CURSOR.y - 10.0);
        let mut render = |ui: &mut egui::Ui| {
            show(ui, &tools, &drawing, &mut camera, &Theme::default());
        };

        harness.frame(vec![moved(CURSOR)], &mut render);
        harness.frame(
            vec![
                moved(CURSOR),
                button(CURSOR, egui::PointerButton::Primary, true),
            ],
            &mut render,
        );
        harness.frame(vec![moved(end)], &mut render);
        harness.frame(
            vec![button(end, egui::PointerButton::Primary, false)],
            render,
        );

        assert_eq!(
            camera.screen_to_world(0.0, 0.0),
            Point { x: 20.0, y: -10.0 }
        );
        assert!(points(&drawing).is_empty());
    }
}
