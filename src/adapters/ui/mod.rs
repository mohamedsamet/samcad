//! Driving adapter: the eframe/egui user interface.

mod app;
mod canvas;
mod cursor;
mod main_bar;
mod theme;
mod tool_panel;
mod top_bar;

#[cfg(test)]
mod test_support;

pub use app::{SamcadApp, native_options};
