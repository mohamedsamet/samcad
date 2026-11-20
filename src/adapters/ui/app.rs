use eframe::egui;

use crate::adapters::window::WindowActionBridge;
use crate::core::ports::{DrawingUseCase, ToolUseCase, WindowUseCase};
use crate::presentation::canvas::Camera;
use crate::presentation::top_bar::TopBarState;

use super::{canvas, main_bar, theme::Theme, tool_panel, top_bar};

/// eframe application: translates user input into use-case calls and renders
/// the result. It contains no business logic.
///
/// All ports use typed protocols for better type safety and testability.
pub struct SamcadApp<T, C, D>
where
    T: WindowUseCase,
    C: ToolUseCase,
    D: DrawingUseCase,
{
    window: T,
    tools: C,
    drawing: D,
    window_bridge: WindowActionBridge,
    theme: Theme,
    top_bar: TopBarState,
    camera: Camera,
}

impl<T, C, D> SamcadApp<T, C, D>
where
    T: WindowUseCase,
    C: ToolUseCase,
    D: DrawingUseCase,
{
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        window: T,
        tools: C,
        drawing: D,
        window_bridge: WindowActionBridge,
    ) -> Self {
        // Required so SVG ImageSource values can be rasterized and displayed.
        egui_extras::install_image_loaders(&cc.egui_ctx);

        Self {
            window,
            tools,
            drawing,
            window_bridge,
            theme: Theme::default(),
            top_bar: TopBarState::default(),
            camera: Camera::default(),
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

impl<T, C, D> eframe::App for SamcadApp<T, C, D>
where
    T: WindowUseCase,
    C: ToolUseCase,
    D: DrawingUseCase,
{
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.window_bridge.apply_to(ui.ctx());

        // Esc clears the active tool, if any.
        if ui.ctx().input(|input| input.key_pressed(egui::Key::Escape)) {
            self.tools.deselect();
        }

        // Ctrl+Shift+Z or Ctrl+Y: Redo (check first — more specific)
        if ui.ctx().input(|input| {
            (input.key_pressed(egui::Key::Y) && input.modifiers.ctrl)
                || (input.key_pressed(egui::Key::Z)
                    && input.modifiers.ctrl
                    && input.modifiers.shift)
        }) {
            self.drawing.redo();
        }
        // Ctrl+Z: Undo
        else if ui
            .ctx()
            .input(|input| input.key_pressed(egui::Key::Z) && input.modifiers.ctrl)
        {
            self.drawing.undo();
        }

        // The main bar reserves space at the top and the tool palette reserves
        // space on the left; the dynamic bar is an overlay drawn on top of them.
        main_bar::show(ui, &self.window, &self.theme);
        tool_panel::show(ui, &self.tools, &self.theme);
        top_bar::show(ui.ctx(), &mut self.top_bar, &self.theme);

        // Drawing canvas, filled last so the panels keep their reserved space.
        canvas::show(
            ui,
            &self.tools,
            &self.drawing,
            &mut self.camera,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::adapters::ui::test_support::{Harness, RecordingWindow, SharedTools, moved, points};
    use crate::adapters::window::WindowActionAdapter;
    use crate::application::drawing::DrawingController;
    use crate::core::model::Tool;
    use eframe::App;
    use egui::Context;

    fn test_app(
        ctx: Context,
        tools: SharedTools,
    ) -> SamcadApp<RecordingWindow, SharedTools, DrawingController> {
        let cc = eframe::CreationContext::_new_kittest(ctx);
        let (_actions, bridge) = WindowActionAdapter::channel();
        SamcadApp::new(
            &cc,
            RecordingWindow::default(),
            tools,
            DrawingController::new(),
            bridge,
        )
    }

    #[test]
    fn native_options_request_a_frameless_fullscreen_window() {
        let options = native_options();

        assert_eq!(options.viewport.app_id.as_deref(), Some("com.samcad.app"));
        assert_eq!(options.viewport.title.as_deref(), Some("samcad"));
        assert_eq!(options.viewport.fullscreen, Some(true));
        assert_eq!(options.viewport.decorations, Some(false));
    }

    #[test]
    fn new_renders_the_whole_ui_headlessly_with_default_state() {
        let ctx = egui::Context::default();
        let tools = SharedTools::default();
        let mut app = test_app(ctx.clone(), tools);
        let mut harness = Harness::with_context(ctx);

        harness.frame(vec![moved(egui::pos2(100.0, 40.0))], |ui| {
            app.ui(ui, &mut eframe::Frame::_new_kittest());
        });

        assert_eq!(app.camera, Camera::default());
        assert_eq!(SharedTools::selected(&app.tools), None);
    }

    #[test]
    fn escape_clears_the_active_tool() {
        let ctx = egui::Context::default();
        let tools = SharedTools::default();
        tools.select(Tool::Line);
        let mut app = test_app(ctx.clone(), tools.clone());
        let mut harness = Harness::with_context(ctx);

        harness.frame(
            vec![egui::Event::Key {
                key: egui::Key::Escape,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Default::default(),
            }],
            |ui| app.ui(ui, &mut eframe::Frame::_new_kittest()),
        );

        assert_eq!(*tools.selected.borrow(), None);
    }

    #[test]
    fn ctrl_z_undoes_last_point() {
        let ctx = egui::Context::default();
        let tools = SharedTools::default();
        tools.select(Tool::Point);
        let mut app = test_app(ctx.clone(), tools);
        let mut harness = Harness::with_context(ctx);

        // Add a point
        harness.click(egui::pos2(100.0, 100.0), |ui| {
            app.ui(ui, &mut eframe::Frame::_new_kittest());
        });
        assert_eq!(points(&app.drawing).len(), 1);

        // Ctrl+Z: undo (press in frame 1, release in frame 2)
        harness.frame(
            vec![
                egui::Event::ModifiersChanged(egui::Modifiers::CTRL),
                egui::Event::Key {
                    key: egui::Key::ControlLeft,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: Default::default(),
                },
                egui::Event::Key {
                    key: egui::Key::Z,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: Default::default(),
                },
            ],
            |ui| app.ui(ui, &mut eframe::Frame::_new_kittest()),
        );
        harness.frame(
            vec![
                egui::Event::Key {
                    key: egui::Key::Z,
                    physical_key: None,
                    pressed: false,
                    repeat: false,
                    modifiers: Default::default(),
                },
                egui::Event::Key {
                    key: egui::Key::ControlLeft,
                    physical_key: None,
                    pressed: false,
                    repeat: false,
                    modifiers: Default::default(),
                },
                egui::Event::ModifiersChanged(egui::Modifiers::NONE),
            ],
            |ui| app.ui(ui, &mut eframe::Frame::_new_kittest()),
        );
        eprintln!("After Ctrl+Z: {} points", points(&app.drawing).len());
        assert_eq!(points(&app.drawing).len(), 0);
    }

    #[test]
    fn ctrl_y_redoes_undone_point() {
        let ctx = egui::Context::default();
        let tools = SharedTools::default();
        tools.select(Tool::Point);
        let mut app = test_app(ctx.clone(), tools);
        let mut harness = Harness::with_context(ctx);

        // Add a point
        harness.click(egui::pos2(100.0, 100.0), |ui| {
            app.ui(ui, &mut eframe::Frame::_new_kittest());
        });
        assert_eq!(points(&app.drawing).len(), 1);

        // Ctrl+Z: undo
        harness.frame(
            vec![
                egui::Event::ModifiersChanged(egui::Modifiers::CTRL),
                egui::Event::Key {
                    key: egui::Key::ControlLeft,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: Default::default(),
                },
                egui::Event::Key {
                    key: egui::Key::Z,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: Default::default(),
                },
            ],
            |ui| app.ui(ui, &mut eframe::Frame::_new_kittest()),
        );
        harness.frame(
            vec![
                egui::Event::Key {
                    key: egui::Key::Z,
                    physical_key: None,
                    pressed: false,
                    repeat: false,
                    modifiers: Default::default(),
                },
                egui::Event::Key {
                    key: egui::Key::ControlLeft,
                    physical_key: None,
                    pressed: false,
                    repeat: false,
                    modifiers: Default::default(),
                },
                egui::Event::ModifiersChanged(egui::Modifiers::NONE),
            ],
            |ui| app.ui(ui, &mut eframe::Frame::_new_kittest()),
        );
        assert_eq!(points(&app.drawing).len(), 0);

        // Ctrl+Y: redo
        harness.frame(
            vec![
                egui::Event::ModifiersChanged(egui::Modifiers::CTRL),
                egui::Event::Key {
                    key: egui::Key::ControlLeft,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: Default::default(),
                },
                egui::Event::Key {
                    key: egui::Key::Y,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: Default::default(),
                },
            ],
            |ui| app.ui(ui, &mut eframe::Frame::_new_kittest()),
        );
        harness.frame(
            vec![
                egui::Event::Key {
                    key: egui::Key::Y,
                    physical_key: None,
                    pressed: false,
                    repeat: false,
                    modifiers: Default::default(),
                },
                egui::Event::Key {
                    key: egui::Key::ControlLeft,
                    physical_key: None,
                    pressed: false,
                    repeat: false,
                    modifiers: Default::default(),
                },
                egui::Event::ModifiersChanged(egui::Modifiers::NONE),
            ],
            |ui| app.ui(ui, &mut eframe::Frame::_new_kittest()),
        );
        assert_eq!(points(&app.drawing).len(), 1);
    }

    #[test]
    fn ctrl_shift_z_redoes_undone_point() {
        let ctx = egui::Context::default();
        let tools = SharedTools::default();
        tools.select(Tool::Point);
        let mut app = test_app(ctx.clone(), tools);
        let mut harness = Harness::with_context(ctx);

        // Add a point
        harness.click(egui::pos2(100.0, 100.0), |ui| {
            app.ui(ui, &mut eframe::Frame::_new_kittest());
        });
        assert_eq!(points(&app.drawing).len(), 1);

        // Ctrl+Z: undo
        harness.frame(
            vec![
                egui::Event::ModifiersChanged(egui::Modifiers::CTRL),
                egui::Event::Key {
                    key: egui::Key::ControlLeft,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: Default::default(),
                },
                egui::Event::Key {
                    key: egui::Key::Z,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: Default::default(),
                },
            ],
            |ui| app.ui(ui, &mut eframe::Frame::_new_kittest()),
        );
        harness.frame(
            vec![
                egui::Event::Key {
                    key: egui::Key::Z,
                    physical_key: None,
                    pressed: false,
                    repeat: false,
                    modifiers: Default::default(),
                },
                egui::Event::Key {
                    key: egui::Key::ControlLeft,
                    physical_key: None,
                    pressed: false,
                    repeat: false,
                    modifiers: Default::default(),
                },
                egui::Event::ModifiersChanged(egui::Modifiers::NONE),
            ],
            |ui| app.ui(ui, &mut eframe::Frame::_new_kittest()),
        );
        assert_eq!(points(&app.drawing).len(), 0);

        // Ctrl+Shift+Z: redo
        harness.frame(
            vec![
                egui::Event::ModifiersChanged(egui::Modifiers::CTRL | egui::Modifiers::SHIFT),
                egui::Event::Key {
                    key: egui::Key::ControlLeft,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: Default::default(),
                },
                egui::Event::Key {
                    key: egui::Key::ShiftLeft,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: Default::default(),
                },
                egui::Event::Key {
                    key: egui::Key::Z,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: Default::default(),
                },
            ],
            |ui| app.ui(ui, &mut eframe::Frame::_new_kittest()),
        );
        harness.frame(
            vec![
                egui::Event::Key {
                    key: egui::Key::Z,
                    physical_key: None,
                    pressed: false,
                    repeat: false,
                    modifiers: Default::default(),
                },
                egui::Event::Key {
                    key: egui::Key::ShiftLeft,
                    physical_key: None,
                    pressed: false,
                    repeat: false,
                    modifiers: Default::default(),
                },
                egui::Event::Key {
                    key: egui::Key::ControlLeft,
                    physical_key: None,
                    pressed: false,
                    repeat: false,
                    modifiers: Default::default(),
                },
                egui::Event::ModifiersChanged(egui::Modifiers::NONE),
            ],
            |ui| app.ui(ui, &mut eframe::Frame::_new_kittest()),
        );
        assert_eq!(points(&app.drawing).len(), 1);
    }
}
