//! Driven adapter for window actions.
//!
//! It converts core requests into messages on a channel. A small bridge on
//! the UI thread drains those messages and turns them into egui viewport
//! commands, so neither the core nor the application layer ever sees egui.

use std::sync::mpsc::{self, Receiver, Sender};

use eframe::egui;

use crate::core::ports::WindowActions;

#[derive(Clone, Copy, Debug)]
enum WindowCommand {
    Minimize,
    Close,
}

/// Implements the outbound `WindowActions` port by queueing commands.
pub struct WindowActionAdapter {
    tx: Sender<WindowCommand>,
}

impl WindowActionAdapter {
    /// Creates the connected adapter (for the core) and bridge (for the UI).
    pub fn channel() -> (Self, WindowActionBridge) {
        let (tx, rx) = mpsc::channel();
        (Self { tx }, WindowActionBridge { rx })
    }
}

impl WindowActions for WindowActionAdapter {
    fn minimize(&self) {
        let _ = self.tx.send(WindowCommand::Minimize);
    }

    fn close(&self) {
        let _ = self.tx.send(WindowCommand::Close);
    }
}

/// UI-side half of the adapter: translates queued commands into viewport
/// commands once per frame.
pub struct WindowActionBridge {
    rx: Receiver<WindowCommand>,
}

impl WindowActionBridge {
    pub fn apply_to(&self, ctx: &egui::Context) {
        while let Ok(command) = self.rx.try_recv() {
            match command {
                WindowCommand::Minimize => {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                }
                WindowCommand::Close => {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
        }
    }
}
