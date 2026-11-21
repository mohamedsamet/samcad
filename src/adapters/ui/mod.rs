//! Driving adapter: the eframe/egui user interface.

mod app;
mod canvas;
mod cursor;
mod dialog;
pub mod file_dialog;
mod main_bar;
mod theme;
mod tool_panel;

#[cfg(test)]
mod test_support;

pub use app::{SamcadApp, native_options};
