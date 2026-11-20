use eframe::egui;

use crate::adapters::config::load_config;
use crate::core::document::Point;
use crate::core::ports::{DrawingUseCase, ToolUseCase};
use crate::presentation::canvas::{self, Camera, CanvasAction, SnapTarget};

use super::cursor;

/// Wheel sensitivity: a notch of ~50 points gives roughly a 10% zoom step.
const ZOOM_SENSITIVITY: f32 = 0.002;

/// Renders the central drawing canvas, the infinite-canvas camera and the CAD
/// cursor.
///
/// Presentation decides what a click means, how world coordinates map to the
/// screen, and which target the cursor is snapped to; this module only feeds
/// pointer input to the camera and paints the visible geometry.
pub fn show(
    ui: &mut egui::Ui,
    tools: &dyn ToolUseCase,
    drawing: &dyn DrawingUseCase,
    camera: &mut Camera,
) {
    // Load adapter config once per frame (cheap; reads from cached file).
    let cfg = load_config();

    egui::CentralPanel::default().show(ui, |ui| {
        let rect = ui.max_rect();
        let response = ui.allocate_rect(rect, egui::Sense::click_and_drag());

        if ui.ui_contains_pointer() {
            ui.ctx().set_cursor_image(Some(cursor::crosshair().clone()));
            // Fallback for integrations that cannot display a bitmap.
            ui.ctx().set_cursor_icon(egui::CursorIcon::Crosshair);
        }

        // Sticky snapping: while the cursor is within the snap radius of an
        // existing target, geometry is placed at that target's exact world
        // position instead of the raw cursor position.
        let snap_target = response
            .hover_pos()
            .map(|pos| pos - rect.min)
            .and_then(|screen| {
                canvas::find_snap_target(
                    drawing,
                    camera,
                    screen.x,
                    screen.y,
                    canvas::SNAP_RADIUS_PX,
                )
            });

        let cursor_world = match snap_target {
            Some(target) => target.world_point(),
            None => response
                .hover_pos()
                .map(|pos| pos - rect.min)
                .map(|local| camera.screen_to_world(local.x, local.y))
                .unwrap_or(Point { x: 0.0, y: 0.0 }),
        };

        update_camera(ui, &response, rect, tools, camera);

        if response.clicked()
            && let Some(action) = canvas::click_action(tools.selected(), cursor_world)
        {
            match action {
                CanvasAction::AddPoint(point) => drawing.add_point(point),
            }
        }

        paint_points(ui, rect, drawing, camera, &cfg.graphics);
        if let Some(target) = snap_target {
            paint_snap_indicator(ui, rect, target, camera, &cfg.graphics);
        }
    });
}

fn paint_snap_indicator(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    target: SnapTarget,
    camera: &Camera,
    graphics: &crate::adapters::config::GraphicsConfig,
) {
    let screen = camera.world_to_screen(&target.world_point());
    if screen.x >= 0.0
        && screen.x <= rect.width()
        && screen.y >= 0.0
        && screen.y <= rect.height()
    {
        // Use the configured snap colour, or fall back to egui's selection
        // colour (the same highlight used for selected buttons).
        let snap_color = graphics.snap_indicator_color;

        ui.painter().circle_stroke(
            rect.min + egui::vec2(screen.x, screen.y),
            6.0,
            egui::Stroke::new(1.5, snap_color),
        );
    }
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
    graphics: &crate::adapters::config::GraphicsConfig,
) {
    let painter = ui.painter();
    let width = rect.width();
    let height = rect.height();
    let margin = graphics.point_radius;

    drawing.for_each_point(&mut |point| {
        let screen = camera.world_to_screen(point);
        if screen.x >= -margin
            && screen.x <= width + margin
            && screen.y >= -margin
            && screen.y <= height + margin
        {
            painter.circle_filled(
                rect.min + egui::vec2(screen.x, screen.y),
                graphics.point_radius,
                graphics.point_color,
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
            show(ui, &tools, &drawing, &mut camera);
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
            show(ui, &tools, &drawing, &mut camera);
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
            show(ui, &tools, &drawing, &mut camera);
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
            show(ui, &tools, &drawing, &mut camera);
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
            show(ui, &tools, &drawing, &mut camera);
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

    #[test]
    fn cursor_snaps_to_point_within_radius() {
        let mut harness = Harness::new();
        let tools = ToolController::new();
        tools.select(Tool::Point);
        let drawing = DrawingController::new();
        let mut camera = Camera::default();

        // Place an existing point directly under the cursor so the click
        // falls within the 10 px snap radius.
        let cursor_world = camera.screen_to_world(
            CURSOR.x - CANVAS_ORIGIN.x,
            CURSOR.y - CANVAS_ORIGIN.y,
        );
        drawing.add_point(cursor_world);

        let mut render = |ui: &mut egui::Ui| {
            show(ui, &tools, &drawing, &mut camera);
        };

        harness.frame(vec![moved(CURSOR)], &mut render);
        harness.click(CURSOR, render);

        // The snapped click lands exactly on the existing point.
        let pts = points(&drawing);
        assert_eq!(pts.len(), 2);
        assert_eq!(pts[0], pts[1]);
    }

    #[test]
    fn snap_is_liberated_when_cursor_moves_far_away() {
        let mut harness = Harness::new();
        let tools = ToolController::new();
        tools.select(Tool::Point);
        let drawing = DrawingController::new();
        let mut camera = Camera::default();

        let existing = Point { x: 0.0, y: 0.0 };
        drawing.add_point(existing);

        let mut render = |ui: &mut egui::Ui| {
            show(ui, &tools, &drawing, &mut camera);
        };

        // The existing point is hundreds of px away: the click is not snapped
        // and lands at the raw cursor world position.
        harness.frame(vec![moved(CURSOR)], &mut render);
        harness.click(CURSOR, render);

        let cursor_world = camera.screen_to_world(
            CURSOR.x - CANVAS_ORIGIN.x,
            CURSOR.y - CANVAS_ORIGIN.y,
        );
        let pts = points(&drawing);
        assert_eq!(pts.len(), 2);
        assert_eq!(pts[1], cursor_world);
        assert_ne!(pts[1], existing);
    }
}
