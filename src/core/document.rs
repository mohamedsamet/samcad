//! Drawing document entities and the document container.
//!
//! Coordinates are stored as `f64`. Until a viewport transform exists they are
//! canvas-local coordinates, so a point survives layout changes but not zoom or
//! pan. A camera will later map them to world/screen space.

/// A 2D point.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

/// The drawing currently being edited.
///
/// For now it only holds points; lines, circles, rectangles and arcs will join
/// it as their tools are implemented.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Document {
    points: Vec<Point>,
}

impl Document {
    /// Appends a point to the drawing.
    pub fn add_point(&mut self, point: Point) {
        self.points.push(point);
    }

    /// The points in the drawing, in insertion order.
    pub fn points(&self) -> &[Point] {
        &self.points
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_empty_and_keeps_points_in_order() {
        let mut document = Document::default();
        assert!(document.points().is_empty());

        document.add_point(Point { x: 1.0, y: 2.0 });
        document.add_point(Point { x: -3.5, y: 4.25 });

        assert_eq!(
            document.points(),
            [Point { x: 1.0, y: 2.0 }, Point { x: -3.5, y: 4.25 },]
        );
    }
}
