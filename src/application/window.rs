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
