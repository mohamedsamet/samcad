//! Domain concepts shared across layers.

/// The drawing tools offered by the toolbar.
///
/// This is domain state: the UI only renders it, and the application layer
/// decides which one is active.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tool {
    Point,
    Line,
    Circle,
    Rectangle,
    Arc,
}

impl Tool {
    pub const ALL: [Self; 5] = [
        Self::Point,
        Self::Line,
        Self::Circle,
        Self::Rectangle,
        Self::Arc,
    ];
}
