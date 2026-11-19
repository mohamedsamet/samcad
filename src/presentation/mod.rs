//! Presentation layer: UI/product decisions expressed as egui-agnostic
//! components.
//!
//! This module builds the descriptions of what should be on screen (which
//! buttons exist, which is selected, what a tooltip says, and eventually what
//! shapes are previewed on the canvas). It never imports egui or eframe.
//! The egui adapter in `adapters/ui` maps these components to widgets.

pub mod canvas;
pub mod component;
pub mod main_bar;
pub mod palette;
pub mod top_bar;
