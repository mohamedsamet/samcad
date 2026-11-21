//! Composition root: wire the adapters and use cases, then launch the UI.
//! No business logic belongs here.

use samcad::adapters::ui::file_dialog::ZenityFileDialog;
use samcad::adapters::ui::{SamcadApp, native_options};
use samcad::adapters::window::WindowActionAdapter;
use samcad::application::{
    drawing::DrawingController, tool::ToolController, window::WindowController,
};

fn main() -> eframe::Result {
    // Driven adapter -> use cases -> driving adapter.
    let (window_actions, window_bridge) = WindowActionAdapter::channel();
    let window = WindowController::new(window_actions);
    let tools = ToolController::new();
    let drawing = DrawingController::new();

    eframe::run_native(
        "samcad",
        native_options(),
        Box::new(move |cc| {
            Ok(Box::new(SamcadApp::new(
                cc,
                window,
                tools,
                drawing,
                window_bridge,
                ZenityFileDialog,
            )))
        }),
    )
}
