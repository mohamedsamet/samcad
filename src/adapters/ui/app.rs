use eframe::egui;

use crate::adapters::window::WindowActionBridge;
use crate::core::ports::{ToolUseCase, WindowUseCase};

use super::{theme::Theme, title_bar, tool_panel};

/// eframe application: translates user input into use-case calls and renders
/// the result. It contains no business logic.
pub struct SamcadApp {
    window: Box<dyn WindowUseCase>,
    tools: Box<dyn ToolUseCase>,
    window_bridge: WindowActionBridge,
    theme: Theme,
}

impl SamcadApp {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        window: Box<dyn WindowUseCase>,
        tools: Box<dyn ToolUseCase>,
        window_bridge: WindowActionBridge,
    ) -> Self {
        // Required so SVG ImageSource values can be rasterized and displayed.
        egui_extras::install_image_loaders(&cc.egui_ctx);

        Self {
            window,
            tools,
            window_bridge,
            theme: Theme::default(),
        }
    }
}

/// Native window configuration belongs to the UI adapter, not to `main`.
pub fn native_options() -> eframe::NativeOptions {
    eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_app_id("com.samcad.app")
            .with_title("samcad")
            .with_fullscreen(true)
            .with_decorations(false),
        ..Default::default()
    }
}

impl eframe::App for SamcadApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.window_bridge.apply_to(ui.ctx());

        title_bar::show(ui, self.window.as_ref());
        tool_panel::show(ui, self.tools.as_ref(), &self.theme);

        // Empty drawing canvas for now.
        egui::CentralPanel::default().show(ui, |_ui| {});
    }
}
