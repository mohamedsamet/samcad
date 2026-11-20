//! Headless egui helpers shared by adapter tests (compiled only under `#[cfg(test)]`).

use std::cell::RefCell;
use std::rc::Rc;

use eframe::egui;

use crate::core::document::Point;
use crate::core::model::Tool;
use crate::core::ports::{DrawingUseCase, ToolUseCase, WindowUseCase};

/// Drives headless egui frames with monotonically increasing time.
pub struct Harness {
    ctx: egui::Context,
    time: f64,
}

impl Harness {
    pub fn new() -> Self {
        Self::with_context(egui::Context::default())
    }

    pub fn with_context(ctx: egui::Context) -> Self {
        // SVG palette icons need the same loaders the real app installs.
        egui_extras::install_image_loaders(&ctx);
        Self { ctx, time: 0.0 }
    }

    /// Renders once to register widget hits, then clicks at `pos` next frame.
    pub fn click(&mut self, pos: egui::Pos2, mut content: impl FnMut(&mut egui::Ui)) {
        self.frame(vec![], &mut content);
        self.frame(click(pos), content);
    }

    pub fn frame(&mut self, events: Vec<egui::Event>, content: impl FnMut(&mut egui::Ui)) {
        let input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1200.0, 800.0),
            )),
            time: Some(self.time),
            events,
            ..Default::default()
        };
        self.time += 1.0;
        let output = self.ctx.run_ui(input, content);
        output.drop_without_applying_deltas();
    }
}

pub fn moved(pos: egui::Pos2) -> egui::Event {
    egui::Event::PointerMoved(pos)
}

pub fn button(pos: egui::Pos2, button: egui::PointerButton, pressed: bool) -> egui::Event {
    egui::Event::PointerButton {
        pos,
        button,
        pressed,
        modifiers: Default::default(),
    }
}

/// A primary press and release at `pos`, in a single frame.
pub fn click(pos: egui::Pos2) -> Vec<egui::Event> {
    vec![
        moved(pos),
        button(pos, egui::PointerButton::Primary, true),
        button(pos, egui::PointerButton::Primary, false),
    ]
}

/// Collects a drawing use case's points without depending on its concrete type.
pub fn points(drawing: &dyn DrawingUseCase) -> Vec<Point> {
    let mut points = Vec::new();
    drawing.for_each_point(&mut |point| points.push(*point));
    points
}

/// Window port mock that records the commands the UI sends.
#[derive(Clone, Default)]
pub struct RecordingWindow {
    pub calls: Rc<RefCell<Vec<&'static str>>>,
}

impl WindowUseCase for RecordingWindow {
    fn minimize(&self) {
        self.calls.borrow_mut().push("minimize");
    }

    fn close(&self) {
        self.calls.borrow_mut().push("close");
    }
}

/// Tool port mock with shared state, so tests can inspect later changes.
#[derive(Clone, Default)]
pub struct SharedTools {
    pub selected: Rc<RefCell<Option<Tool>>>,
}

impl ToolUseCase for SharedTools {
    fn select(&self, tool: Tool) {
        *self.selected.borrow_mut() = Some(tool);
    }

    fn deselect(&self) {
        *self.selected.borrow_mut() = None;
    }

    fn selected(&self) -> Option<Tool> {
        *self.selected.borrow()
    }
}
