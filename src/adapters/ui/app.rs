use eframe::egui;

use crate::adapters::window::WindowActionBridge;
use crate::core::ports::{ToolUseCase, WindowUseCase};
use crate::presentation::top_bar::TopBarState;

use super::{canvas, main_bar, theme::Theme, tool_panel, top_bar};

/// eframe application: translates user input into use-case calls and renders
/// the result. It contains no business logic.
pub struct SamcadApp {
    window: Box<dyn WindowUseCase>,
    tools: Box<dyn ToolUseCase>,
    window_bridge: WindowActionBridge,
    theme: Theme,
    top_bar: TopBarState,
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
            top_bar: TopBarState::default(),
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

        // The main bar reserves space at the top and the tool palette reserves
        // space on the left; the dynamic bar is an overlay drawn on top of them.
        main_bar::show(ui, self.window.as_ref(), &self.theme);
        tool_panel::show(ui, self.tools.as_ref(), &self.theme);
        top_bar::show(ui.ctx(), &mut self.top_bar, &self.theme);

        // Drawing canvas, filled last so the panels keep their reserved space.
        canvas::show(ui);
    }
}
