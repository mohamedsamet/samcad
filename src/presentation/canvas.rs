//! Presentation logic for the drawing canvas.
//!
//! Two decisions are cooked here, with no egui dependency:
//! 1. What a click on the canvas means for the active tool.
//! 2. How document points become paint-ready canvas primitives.

use crate::core::document::Point;
use crate::core::model::Tool;

/// What a click on the canvas should do, given the active tool.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CanvasAction {
    AddPoint { x: f64, y: f64 },
}

/// Decides the action for a click at canvas-local `(x, y)`.
///
/// Only the Point tool creates geometry for now; other tools will gain their
/// own actions as they are implemented.
pub fn click_action(tool: Option<Tool>, x: f32, y: f32) -> Option<CanvasAction> {
    match tool {
        Some(Tool::Point) => Some(CanvasAction::AddPoint {
            x: f64::from(x),
            y: f64::from(y),
        }),
        _ => None,
    }
}

/// A point ready to be painted, in canvas-local coordinates.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CanvasPoint {
    pub x: f32,
    pub y: f32,
}

/// Cooks document points into paint-ready canvas points.
pub fn view_model(points: &[Point]) -> impl Iterator<Item = CanvasPoint> + '_ {
    points.iter().map(|point| CanvasPoint {
        x: point.x as f32,
        y: point.y as f32,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn point_tool_click_adds_a_point_at_the_click_position() {
        assert_eq!(
            click_action(Some(Tool::Point), 12.5, 34.0),
            Some(CanvasAction::AddPoint { x: 12.5, y: 34.0 })
        );
    }

    #[test]
    fn clicking_with_another_tool_or_no_tool_does_nothing() {
        assert_eq!(click_action(None, 1.0, 2.0), None);
        assert_eq!(click_action(Some(Tool::Line), 1.0, 2.0), None);
        assert_eq!(click_action(Some(Tool::Circle), 1.0, 2.0), None);
    }

    #[test]
    fn view_model_maps_document_points_to_canvas_points() {
        let points = [Point { x: 1.0, y: 2.0 }, Point { x: -3.5, y: 4.25 }];

        let canvas: Vec<CanvasPoint> = view_model(&points).collect();

        assert_eq!(
            canvas,
            [
                CanvasPoint { x: 1.0, y: 2.0 },
                CanvasPoint { x: -3.5, y: 4.25 },
            ]
        );
    }
}
