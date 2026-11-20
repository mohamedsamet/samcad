use std::cell::RefCell;

use crate::core::command::{AddPointCommand, CommandHistory};
use crate::core::document::{Document, Point};
use crate::core::ports::DrawingUseCase;

/// Use case that owns the drawing being edited.
///
/// State lives here, not in the UI adapter, so the same document can later be
/// driven by other adapters (tests, CLI). Interior mutability matches the
/// read-only `&self` driving ports. Commands are used to support undo/redo.
pub struct DrawingController {
    document: RefCell<Document>,
    history: RefCell<CommandHistory>,
}

impl DrawingController {
    pub fn new() -> Self {
        Self {
            document: RefCell::new(Document::default()),
            history: RefCell::new(CommandHistory::default()),
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
        let mut document = self.document.borrow_mut();
        let mut history = self.history.borrow_mut();
        let command = Box::new(AddPointCommand::new(point));
        history.execute(command, &mut document);
    }

    fn for_each_point(&self, visit: &mut dyn FnMut(&Point)) {
        for point in self.document.borrow().points() {
            visit(point);
        }
    }

    fn clear(&self) {
        *self.document.borrow_mut() = Document::default();
        self.history.borrow_mut().clear();
    }

    fn undo(&self) -> bool {
        let mut history = self.history.borrow_mut();
        history.undo(&mut self.document.borrow_mut())
    }

    fn redo(&self) -> bool {
        let mut history = self.history.borrow_mut();
        history.redo(&mut self.document.borrow_mut())
    }

    fn can_undo(&self) -> bool {
        self.history.borrow().can_undo()
    }

    fn can_redo(&self) -> bool {
        self.history.borrow().can_redo()
    }
}
