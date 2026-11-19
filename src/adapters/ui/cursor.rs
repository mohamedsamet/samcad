//! CAD-style bitmap cursor for the working area.
//!
//! The bitmap is embedded as raw RGBA (generated from `pointer.jpeg`), so no
//! runtime image-decoding dependency is needed. egui-winit dedupes custom
//! cursors by `Arc` pointer, hence the shared cached instance.

use std::sync::{Arc, OnceLock};

use eframe::egui;

const CURSOR_RGBA: &[u8] = include_bytes!("icons/cursor_crosshair.rgba");
const CURSOR_SIZE: u16 = 68;

static CROSSHAIR: OnceLock<egui::CustomCursorImage> = OnceLock::new();

/// The crosshair-with-pickbox cursor shown over the drawing canvas.
pub fn crosshair() -> &'static egui::CustomCursorImage {
    CROSSHAIR.get_or_init(|| egui::CustomCursorImage {
        rgba: Arc::from(CURSOR_RGBA),
        size: [CURSOR_SIZE, CURSOR_SIZE],
        hotspot: [CURSOR_SIZE / 2, CURSOR_SIZE / 2],
    })
}
