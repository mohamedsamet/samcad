//! Command pattern for undo/redo operations.
//!
//! Every mutation is represented as a Command that can be executed and undone.
//! The CommandHistory owns undo/redo stacks.

use super::document::{Document, Point};

/// Any operation that mutates the document.
///
/// Commands are self-contained and contain all information needed to both
/// execute and undo the change.
pub trait Command {
    /// Apply the command to the document - mutates it directly.
    fn execute(&mut self, document: &mut Document);

    /// Revert the command - restores the document to its previous state.
    fn undo(&mut self, document: &mut Document);
}

/// History of undo/redo operations.
///
/// Tracks all commands in a stack, allowing backtracking through the editor's
/// state changes.
#[derive(Default)]
pub struct CommandHistory {
    undo_stack: Vec<Box<dyn Command>>,
    redo_stack: Vec<Box<dyn Command>>,
}

impl CommandHistory {
    /// Execute a command, adding it to the undo stack.
    pub fn execute(&mut self, mut command: Box<dyn Command>, document: &mut Document) {
        command.execute(document);
        self.undo_stack.push(command);
        self.redo_stack.clear();
    }

    /// Undo the last command.
    pub fn undo(&mut self, document: &mut Document) -> bool {
        if let Some(mut command) = self.undo_stack.pop() {
            command.undo(document);
            self.redo_stack.push(command);
            true
        } else {
            false
        }
    }

    /// Redo the last undone command.
    pub fn redo(&mut self, document: &mut Document) -> bool {
        if let Some(mut command) = self.redo_stack.pop() {
            command.execute(document);
            self.undo_stack.push(command);
            true
        } else {
            false
        }
    }

    /// Is there something to undo?
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Is there something to redo?
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Clear all history.
    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }
}

/// Add a point command.
pub struct AddPointCommand {
    point: Point,
    index: usize,
}

impl AddPointCommand {
    pub fn new(point: Point) -> Self {
        Self { point, index: 0 }
    }
}

impl Command for AddPointCommand {
    fn execute(&mut self, document: &mut Document) {
        self.index = document.points().len();
        document.add_point(self.point);
    }

    fn undo(&mut self, document: &mut Document) {
        document.remove_point(self.index);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_history_undo_redo() {
        let mut doc = Document::default();
        let mut history = CommandHistory::default();

        let cmd1 = Box::new(AddPointCommand::new(Point { x: 1.0, y: 1.0 }));
        let cmd2 = Box::new(AddPointCommand::new(Point { x: 2.0, y: 2.0 }));

        history.execute(cmd1, &mut doc);
        history.execute(cmd2, &mut doc);
        assert_eq!(doc.points().len(), 2);

        history.undo(&mut doc);
        assert_eq!(doc.points().len(), 1);
        assert!(history.can_redo());

        history.redo(&mut doc);
        assert_eq!(doc.points().len(), 2);
    }

    #[test]
    fn command_history_clear() {
        let mut doc = Document::default();
        let mut history = CommandHistory::default();

        history.execute(
            Box::new(AddPointCommand::new(Point { x: 1.0, y: 1.0 })),
            &mut doc,
        );
        history.clear();
        assert!(!history.can_undo());
    }
}
