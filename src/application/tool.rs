use std::cell::Cell;

use crate::core::model::Tool;
use crate::core::ports::ToolUseCase;

/// Use case that keeps track of the active drawing tool.
///
/// No tool is active until the user picks one. State lives here, not in the
/// UI adapter, so tools can later be driven by other adapters too.
pub struct ToolController {
    selected: Cell<Option<Tool>>,
}

impl ToolController {
    pub fn new() -> Self {
        Self {
            selected: Cell::new(None),
        }
    }
}

impl Default for ToolController {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolUseCase for ToolController {
    fn select(&self, tool: Tool) {
        self.selected.set(Some(tool));
    }

    fn deselect(&self) {
        self.selected.set(None);
    }

    fn selected(&self) -> Option<Tool> {
        self.selected.get()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tracks_the_selected_tool() {
        let tools = ToolController::new();
        assert_eq!(tools.selected(), None);

        tools.select(Tool::Circle);
        assert_eq!(tools.selected(), Some(Tool::Circle));

        tools.select(Tool::Arc);
        assert_eq!(tools.selected(), Some(Tool::Arc));
    }

    #[test]
    fn deselect_clears_the_active_tool() {
        let tools = ToolController::new();
        tools.select(Tool::Line);

        tools.deselect();
        assert_eq!(tools.selected(), None);

        // Deselecting again is a no-op.
        tools.deselect();
        assert_eq!(tools.selected(), None);
    }
}
