//! Port trait definitions (the boundaries of the hexagon).

use crate::core::model::Tool;

/// Driving port (inbound): what a caller, such as the UI, may ask the
/// application to do.
///
/// Implemented by an application use case, never by an adapter.
pub trait WindowUseCase {
    fn minimize(&self);
    fn close(&self);
}

/// Driving port for tool selection.
pub trait ToolUseCase {
    fn select(&self, tool: Tool);
    fn selected(&self) -> Option<Tool>;
}

/// Driven port (outbound): a capability the application needs from the
/// outside world.
///
/// Implemented by an adapter; the core only knows this trait.
pub trait WindowActions {
    fn minimize(&self);
    fn close(&self);
}
