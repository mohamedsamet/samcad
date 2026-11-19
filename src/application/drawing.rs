use std::cell::RefCell;

use crate::core::document::{Document, Point};
use crate::core::ports::DrawingUseCase;

/// Use case that owns the drawing being edited.
///
/// State lives here, not in the UI adapter, so the same document can later be
/// driven by other adapters (tests, CLI). Interior mutability matches the
/// read-only `&self` driving ports.
pub struct DrawingController {
    document: RefCell<Document>,
}

impl DrawingController {
    pub fn new() -> Self {
        Self {
            document: RefCell::new(Document::default()),
        }
    }
}

impl Default for DrawingController {
    fn default() -> Self {
        Self::new()
    }
}

impl DrawingUseCase for DrawingController {
    fn add_point(&self, point: Point) {
        self.document.borrow_mut().add_point(point);
    }

    fn for_each_point(&self, visit: &mut dyn FnMut(&Point)) {
        for point in self.document.borrow().points() {
            visit(point);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn points(drawing: &DrawingController) -> Vec<Point> {
        let mut points = Vec::new();
        drawing.for_each_point(&mut |point| points.push(*point));
        points
    }

    #[test]
    fn adds_points_to_the_document() {
        let drawing = DrawingController::new();
        assert!(points(&drawing).is_empty());

        drawing.add_point(Point { x: 10.0, y: 20.0 });
        drawing.add_point(Point { x: 30.0, y: 40.0 });

        assert_eq!(
            points(&drawing),
            [Point { x: 10.0, y: 20.0 }, Point { x: 30.0, y: 40.0 }]
        );
    }

    #[test]
    fn default_controller_starts_empty() {
        let drawing = DrawingController::default();
        assert!(points(&drawing).is_empty());
    }
}
