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

    fn points(&self) -> Vec<Point> {
        self.document.borrow().points().to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adds_points_to_the_document() {
        let drawing = DrawingController::new();
        assert!(drawing.points().is_empty());

        drawing.add_point(Point { x: 10.0, y: 20.0 });
        drawing.add_point(Point { x: 30.0, y: 40.0 });

        assert_eq!(
            drawing.points(),
            [Point { x: 10.0, y: 20.0 }, Point { x: 30.0, y: 40.0 }]
        );
    }
}
