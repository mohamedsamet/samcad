//! Drawing document entities and the document container.
//!
//! Coordinates are stored as `f64` in drawing/world units. Presentation maps
//! them to canvas-local screen points through its camera, so a point survives
//! layout changes, zooming and panning.

use crate::core::format::{DocumentData, PointData};

/// A 2D point.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl From<Point> for PointData {
    fn from(p: Point) -> Self {
        PointData { x: p.x, y: p.y }
    }
}

impl From<PointData> for Point {
    fn from(p: PointData) -> Self {
        Point { x: p.x, y: p.y }
    }
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

    /// Removes a point at a specific index.
    pub fn remove_point(&mut self, index: usize) -> Option<Point> {
        if index < self.points.len() {
            Some(self.points.remove(index))
        } else {
            None
        }
    }

    /// Inserts a point at a specific index.
    pub fn insert_point(&mut self, index: usize, point: Point) {
        if index <= self.points.len() {
            self.points.insert(index, point);
        } else {
            self.points.push(point);
        }
    }

    /// The points in the drawing, in insertion order.
    pub fn points(&self) -> &[Point] {
        &self.points
    }

    /// Convert the document to serializable data.
    pub fn to_data(&self) -> DocumentData {
        DocumentData {
            version: DocumentData::CURRENT_VERSION,
            points: self.points.iter().map(|p| (*p).into()).collect(),
        }
    }

    /// Create a document from serializable data.
    pub fn from_data(data: DocumentData) -> Self {
        Self {
            points: data.points.into_iter().map(|p| p.into()).collect(),
        }
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
