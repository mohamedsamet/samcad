use crate::core::ports::{WindowActions, WindowUseCase};

/// Use case for controlling the application window.
///
/// It holds no framework code: requests from the driving adapter are simply
/// forwarded to the configured driven port.
pub struct WindowController<A: WindowActions> {
    actions: A,
}

impl<A: WindowActions> WindowController<A> {
    pub fn new(actions: A) -> Self {
        Self { actions }
    }
}

impl<A: WindowActions> WindowUseCase for WindowController<A> {
    fn minimize(&self) {
        self.actions.minimize();
    }

    fn close(&self) {
        self.actions.close();
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use super::*;

    #[derive(Clone, Default)]
    struct RecordingActions {
        calls: Rc<RefCell<Vec<&'static str>>>,
    }

    impl WindowActions for RecordingActions {
        fn minimize(&self) {
            self.calls.borrow_mut().push("minimize");
        }

        fn close(&self) {
            self.calls.borrow_mut().push("close");
        }
    }

    #[test]
    fn forwards_window_commands_to_the_driven_port() {
        let actions = RecordingActions::default();
        let window = WindowController::new(actions.clone());

        window.minimize();
        window.close();

        assert_eq!(*actions.calls.borrow(), ["minimize", "close"]);
    }
}
