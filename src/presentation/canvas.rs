//! Presentation logic for the drawing canvas: the infinite-canvas camera and
//! the meaning of a click. Nothing here depends on egui.
//!
//! The camera is the only state that maps world coordinates (f64, unbounded)
//! to canvas-local screen points (f32). Zoom and pan mutate it; painting only
//! reads it.

use crate::core::document::Point;
use crate::core::model::Tool;

/// Smallest allowed zoom, in screen points per world unit.
pub const MIN_ZOOM: f32 = 0.01;
/// Largest allowed zoom, in screen points per world unit.
pub const MAX_ZOOM: f32 = 10_000.0;

/// A position in canvas-local screen points (origin at the canvas top-left).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScreenPoint {
    pub x: f32,
    pub y: f32,
}

/// An infinite-canvas camera.
///
/// `origin` is the world coordinate shown at the canvas top-left; `zoom` is how
/// many screen points one world unit occupies. World Y points up (CAD/DXF
/// convention) while screen Y points down, so the conversion flips Y. The
/// default view is 1:1 with world (0,0) at the top-left.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Camera {
    origin: Point,
    zoom: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            origin: Point { x: 0.0, y: 0.0 },
            zoom: 1.0,
        }
    }
}

impl Camera {
    pub fn zoom(&self) -> f32 {
        self.zoom
    }

    /// Turns canvas-local screen coordinates into world coordinates.
    pub fn screen_to_world(&self, x: f32, y: f32) -> Point {
        let zoom = f64::from(self.zoom);
        Point {
            x: self.origin.x + f64::from(x) / zoom,
            y: self.origin.y - f64::from(y) / zoom,
        }
    }

    /// Turns a world point into canvas-local screen coordinates.
    pub fn world_to_screen(&self, point: &Point) -> ScreenPoint {
        let zoom = f64::from(self.zoom);
        ScreenPoint {
            x: ((point.x - self.origin.x) * zoom) as f32,
            y: ((self.origin.y - point.y) * zoom) as f32,
        }
    }

    /// Zooms by `factor` while keeping the world point under screen `(x, y)`
    /// fixed, so wheel zoom feels anchored to the cursor.
    pub fn zoom_around(&mut self, x: f32, y: f32, factor: f32) {
        if !factor.is_finite() || factor <= 0.0 {
            return;
        }

        let anchor = self.screen_to_world(x, y);
        let new_zoom = (self.zoom * factor).clamp(MIN_ZOOM, MAX_ZOOM);
        if new_zoom == self.zoom {
            return;
        }

        self.zoom = new_zoom;
        let zoom = f64::from(new_zoom);
        self.origin = Point {
            x: anchor.x - f64::from(x) / zoom,
            y: anchor.y + f64::from(y) / zoom,
        };
    }

    /// Pans by a screen-space delta, e.g. the pointer movement while dragging.
    pub fn pan_by(&mut self, dx: f32, dy: f32) {
        let zoom = f64::from(self.zoom);
        self.origin.x -= f64::from(dx) / zoom;
        self.origin.y += f64::from(dy) / zoom;
    }
}

/// What a click on the canvas should do, given the active tool.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CanvasAction {
    AddPoint(Point),
}

/// Decides the action for a primary click at world `point`.
///
/// Only the Point tool creates geometry for now; other tools will gain their
/// own actions as they are implemented.
pub fn click_action(tool: Option<Tool>, point: Point) -> Option<CanvasAction> {
    matches!(tool, Some(Tool::Point)).then_some(CanvasAction::AddPoint(point))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_close(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() < 1e-3,
            "expected {expected}, got {actual}"
        );
    }

    #[test]
    fn default_camera_is_one_to_one() {
        let camera = Camera::default();

        assert_eq!(
            camera.screen_to_world(10.0, 20.0),
            Point { x: 10.0, y: -20.0 }
        );
        let screen = camera.world_to_screen(&Point { x: 10.0, y: -20.0 });
        assert_eq!(screen, ScreenPoint { x: 10.0, y: 20.0 });
    }

    #[test]
    fn screen_and_world_round_trip_after_zoom_and_pan() {
        let mut camera = Camera::default();
        camera.pan_by(-40.0, 25.0);
        camera.zoom_around(100.0, 80.0, 2.5);

        let screen = ScreenPoint { x: 123.0, y: 45.0 };
        let world = camera.screen_to_world(screen.x, screen.y);
        let back = camera.world_to_screen(&world);

        assert_close(back.x, screen.x);
        assert_close(back.y, screen.y);
    }

    #[test]
    fn zoom_keeps_the_world_point_under_the_cursor_fixed() {
        let mut camera = Camera::default();
        let anchor = ScreenPoint { x: 200.0, y: 120.0 };
        let before = camera.screen_to_world(anchor.x, anchor.y);

        camera.zoom_around(anchor.x, anchor.y, 4.0);

        let after = camera.screen_to_world(anchor.x, anchor.y);
        assert_close(after.x as f32, before.x as f32);
        assert_close(after.y as f32, before.y as f32);
        assert_close(camera.zoom(), 4.0);
    }

    #[test]
    fn zoom_is_clamped() {
        let mut camera = Camera::default();

        camera.zoom_around(0.0, 0.0, 1e12);
        assert_eq!(camera.zoom(), MAX_ZOOM);

        camera.zoom_around(0.0, 0.0, 1e-12);
        assert_eq!(camera.zoom(), MIN_ZOOM);
    }

    #[test]
    fn invalid_zoom_factors_are_ignored() {
        let mut camera = Camera::default();
        let anchor = ScreenPoint { x: 40.0, y: 30.0 };
        let before = camera.screen_to_world(anchor.x, anchor.y);

        for factor in [f32::NAN, f32::INFINITY, 0.0, -2.0] {
            camera.zoom_around(anchor.x, anchor.y, factor);
        }

        assert_eq!(camera.zoom(), 1.0);
        assert_eq!(camera.screen_to_world(anchor.x, anchor.y), before);
    }

    #[test]
    fn zooming_at_the_limit_is_a_no_op() {
        let mut camera = Camera::default();
        camera.zoom_around(0.0, 0.0, MAX_ZOOM);
        assert_eq!(camera.zoom(), MAX_ZOOM);

        // Still clamped to MAX_ZOOM, so the camera must not move.
        let before = camera.screen_to_world(10.0, 10.0);
        camera.zoom_around(10.0, 10.0, 2.0);
        assert_eq!(camera.zoom(), MAX_ZOOM);
        assert_eq!(camera.screen_to_world(10.0, 10.0), before);
    }

    #[test]
    fn pan_moves_content_with_the_drag() {
        let mut camera = Camera::default();

        // Dragging right by 10 px moves the view 10 world units left; dragging
        // up by 5 px (screen Y down) moves the view 5 world units down.
        camera.pan_by(10.0, -5.0);

        assert_eq!(
            camera.screen_to_world(0.0, 0.0),
            Point { x: -10.0, y: -5.0 }
        );
    }

    #[test]
    fn point_tool_click_adds_a_point_at_the_click_world_position() {
        let point = Point { x: 12.5, y: 34.0 };
        assert_eq!(
            click_action(Some(Tool::Point), point),
            Some(CanvasAction::AddPoint(point))
        );
    }

    #[test]
    fn clicking_with_another_tool_or_no_tool_does_nothing() {
        let point = Point { x: 1.0, y: 2.0 };
        assert_eq!(click_action(None, point), None);
        assert_eq!(click_action(Some(Tool::Line), point), None);
        assert_eq!(click_action(Some(Tool::Circle), point), None);
    }
}
