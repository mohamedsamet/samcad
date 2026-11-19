use eframe::egui;

use crate::adapters::window::WindowActionBridge;
use crate::core::ports::{ToolUseCase, WindowUseCase};
use crate::presentation::top_bar::TopBarState;

use super::{cursor, main_bar, theme::Theme, tool_panel, top_bar};

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

        // The permanent bar reserves space; the dynamic one overlays just
        // below it, so neither covers the tool palette.
        main_bar::show(ui, self.window.as_ref(), &self.theme);
        tool_panel::show(ui, self.tools.as_ref(), &self.theme);
        top_bar::show(ui.ctx(), &mut self.top_bar, &self.theme);

        // Empty drawing canvas for now. Over the working zone the pointer
        // becomes the CAD-style bitmap crosshair from pointer.jpeg; elsewhere
        // widgets keep their normal cursor.
        egui::CentralPanel::default().show(ui, |ui| {
            if ui.ui_contains_pointer() {
                ui.ctx().set_cursor_image(Some(cursor::crosshair().clone()));
                // Fallback for integrations that cannot display a bitmap.
                ui.ctx().set_cursor_icon(egui::CursorIcon::Crosshair);
            }
        });
    }
}
