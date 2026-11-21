use eframe::egui;

use crate::adapters::ui::file_dialog::FileDialog;
use crate::adapters::window::WindowActionBridge;
use crate::application::load::{LoadController, LoadUseCase, StdFileSystem as LoadStdFileSystem};
use crate::application::save::{SaveController, SaveUseCase, StdFileSystem};
use crate::core::ports::{DrawingUseCase, ToolUseCase, WindowUseCase};
use crate::presentation::canvas::Camera;
use crate::presentation::component::MenuAction;
use crate::presentation::dialog::DialogState;
use crate::presentation::new_project;
use crate::presentation::open_project;

use super::{canvas, dialog, main_bar, theme::Theme, tool_panel};

/// eframe application: translates user input into use-case calls and renders
/// the result. It contains no business logic.
///
/// All ports use typed protocols for better type safety and testability.
pub struct SamcadApp<T, C, D, F>
where
    T: WindowUseCase,
    C: ToolUseCase,
    D: DrawingUseCase,
    F: FileDialog,
{
    window: T,
    tools: C,
    drawing: D,
    window_bridge: WindowActionBridge,
    theme: Theme,
    new_project_dialog: DialogState,
    open_project_dialog: DialogState,
    pending_open_path: Option<std::path::PathBuf>,
    camera: Camera,
    current_path: Option<std::path::PathBuf>,
    file_dialog: F,
}

impl<T, C, D, F> SamcadApp<T, C, D, F>
where
    T: WindowUseCase,
    C: ToolUseCase,
    D: DrawingUseCase,
    F: FileDialog,
{
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        window: T,
        tools: C,
        drawing: D,
        window_bridge: WindowActionBridge,
        file_dialog: F,
    ) -> Self {
        // Required so SVG ImageSource values can be rasterized and displayed.
        egui_extras::install_image_loaders(&cc.egui_ctx);

        Self {
            window,
            tools,
            drawing,
            window_bridge,
            theme: Theme::default(),
            new_project_dialog: DialogState::default(),
            open_project_dialog: DialogState::default(),
            pending_open_path: None,
            camera: Camera::default(),
            current_path: None,
            file_dialog,
        }
    }

    fn document_title(&self) -> Option<String> {
        self.current_path
            .as_ref()
            .and_then(|path| path.file_stem())
            .and_then(|stem| stem.to_str())
            .map(str::to_string)
    }

    fn clear_project(&mut self) {
        self.drawing.clear();
        self.drawing.mark_saved();
        self.camera = Camera::default();
        self.current_path = None;
    }

    fn new_project(&mut self) {
        if !self.drawing.is_dirty() {
            self.clear_project();
            return;
        }
        self.new_project_dialog.open();
    }

    fn save_as(&mut self) -> bool {
        match self.file_dialog.save_dialog("untitled.samcad") {
            Some(path) => self.save_to(path),
            None => false,
        }
    }

    fn save_as_new(&mut self) -> bool {
        match self.file_dialog.save_new_dialog("untitled.samcad") {
            Some(path) => self.save_to(path),
            None => false,
        }
    }

    fn save(&mut self) -> bool {
        match self.current_path.clone() {
            Some(path) => self.save_to(path),
            None => self.save_as(),
        }
    }

    fn save_to(&mut self, path: std::path::PathBuf) -> bool {
        let saver = SaveController::new(StdFileSystem);
        match saver.save_as(path.clone(), &self.drawing, &self.camera) {
            Ok(()) => {
                self.current_path = Some(path);
                self.drawing.mark_saved();
                true
            }
            Err(e) => {
                eprintln!("Failed to save: {}", e);
                false
            }
        }
    }

    fn open(&mut self) {
        if let Some(path) = self.file_dialog.open_dialog() {
            if !self.drawing.is_dirty() {
                self.load_file(path);
                return;
            }
            self.pending_open_path = Some(path);
            self.open_project_dialog.open();
        }
    }

    fn load_file(&mut self, path: std::path::PathBuf) {
        let loader = LoadController::new(LoadStdFileSystem);
        match loader.load(path.clone()) {
            Ok(result) => {
                self.drawing.clear();
                for point in result.document.points() {
                    self.drawing.add_point(*point);
                }
                self.camera = result.camera;
                self.current_path = Some(path);
                self.drawing.mark_saved();
            }
            Err(e) => eprintln!("Failed to load: {}", e),
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

impl<T, C, D, F> eframe::App for SamcadApp<T, C, D, F>
where
    T: WindowUseCase,
    C: ToolUseCase,
    D: DrawingUseCase,
    F: FileDialog,
{
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.window_bridge.apply_to(ui.ctx());

        // A modal dialog owns the keyboard while it is open.
        let dialog_open = self.new_project_dialog.is_open();

        // Esc clears the active tool, if any (unless a dialog is open).
        if !dialog_open && ui.ctx().input(|input| input.key_pressed(egui::Key::Escape)) {
            self.tools.deselect();
        }

        // Ctrl+Shift+Z or Ctrl+Y: Redo (check first — more specific)
        if !dialog_open
            && ui.ctx().input(|input| {
                (input.key_pressed(egui::Key::Y) && input.modifiers.ctrl)
                    || (input.key_pressed(egui::Key::Z)
                        && input.modifiers.ctrl
                        && input.modifiers.shift)
            })
        {
            self.drawing.redo();
        }
        // Ctrl+Z: Undo
        else if !dialog_open
            && ui
                .ctx()
                .input(|input| input.key_pressed(egui::Key::Z) && input.modifiers.ctrl)
        {
            self.drawing.undo();
        }

        // Ctrl+S: Save
        if !dialog_open
            && ui.ctx().input(|input| {
                input.key_pressed(egui::Key::S) && input.modifiers.ctrl && !input.modifiers.shift
            })
        {
            self.save();
        }
        // Ctrl+Shift+S: Save As
        else if !dialog_open
            && ui.ctx().input(|input| {
                input.key_pressed(egui::Key::S) && input.modifiers.ctrl && input.modifiers.shift
            })
        {
            self.save_as_new();
        }
        // Ctrl+O: Open
        else if !dialog_open
            && ui
                .ctx()
                .input(|input| input.key_pressed(egui::Key::O) && input.modifiers.ctrl)
        {
            self.open();
        }

        // The main bar reserves space at the top and the tool palette reserves
        // space on the left.
        let document_title = self.document_title();
        let model = crate::presentation::main_bar::view_model(
            document_title.as_deref(),
            self.drawing.is_dirty(),
        );
        if let Some(action) = main_bar::show(ui, &self.window, &self.theme, &model) {
            match action {
                MenuAction::NewProject => self.new_project(),
                MenuAction::Open => self.open(),
                MenuAction::Save => {
                    self.save();
                }
                MenuAction::SaveAs => {
                    self.save_as_new();
                }
                MenuAction::Export => {}
            }
        }
        tool_panel::show(ui, &self.tools, &self.theme);

        // Drawing canvas, filled last so the panels keep their reserved space.
        canvas::show(ui, &self.tools, &self.drawing, &mut self.camera);

        // Modal dialogs paint over everything else. "Save" persists the current
        // drawing before starting a new project; "Don't Save" discards it.
        let new_project_model = new_project::dialog_model();
        if let Some(choice) = dialog::show(
            ui.ctx(),
            egui::Id::unique("new_project_dialog"),
            &mut self.new_project_dialog,
            &new_project_model,
            &self.theme,
        ) {
            match choice {
                new_project::NewProjectChoice::Save => {
                    if self.save() {
                        self.clear_project();
                    }
                }
                new_project::NewProjectChoice::Discard => self.clear_project(),
            }
        }

        // Open file dialog: same pattern when there are unsaved changes.
        let open_project_model = open_project::dialog_model();
        if let Some(choice) = dialog::show(
            ui.ctx(),
            egui::Id::unique("open_project_dialog"),
            &mut self.open_project_dialog,
            &open_project_model,
            &self.theme,
        ) {
            if let Some(path) = self.pending_open_path.take() {
                match choice {
                    open_project::OpenProjectChoice::Save => {
                        if self.save() {
                            self.load_file(path);
                        }
                    }
                    open_project::OpenProjectChoice::Discard => self.load_file(path),
                    open_project::OpenProjectChoice::Cancel => {}
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::adapters::ui::file_dialog::MockFileDialog;
    use crate::adapters::ui::test_support::{Harness, RecordingWindow, SharedTools, points};
    use crate::adapters::window::WindowActionAdapter;
    use crate::application::drawing::DrawingController;
    use crate::core::model::Tool;
    use eframe::App;
    use egui::Context;
    use std::path::PathBuf;

    fn test_app(
        ctx: Context,
        tools: SharedTools,
        file_dialog: MockFileDialog,
    ) -> SamcadApp<RecordingWindow, SharedTools, DrawingController, MockFileDialog> {
        let cc = eframe::CreationContext::_new_kittest(ctx);
        let (_actions, bridge) = WindowActionAdapter::channel();
        SamcadApp::new(
            &cc,
            RecordingWindow::default(),
            tools,
            DrawingController::new(),
            bridge,
            file_dialog,
        )
    }

    fn default_test_app(
        ctx: Context,
        tools: SharedTools,
    ) -> SamcadApp<RecordingWindow, SharedTools, DrawingController, MockFileDialog> {
        test_app(ctx, tools, MockFileDialog::default())
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
        let mut app = default_test_app(ctx.clone(), tools);
        let mut harness = Harness::with_context(ctx);

        harness.frame(vec![], |ui| {
            app.ui(ui, &mut eframe::Frame::_new_kittest());
        });

        assert_eq!(app.camera, Camera::default());
        assert_eq!(SharedTools::selected(&app.tools), None);
    }

    #[test]
    fn document_title_extracts_filename_without_extension() {
        let ctx = egui::Context::default();
        let tools = SharedTools::default();
        let mut app = default_test_app(ctx, tools);

        // No path set
        assert_eq!(app.document_title(), None);

        // Set a path
        app.current_path = Some(PathBuf::from("/home/user/drawing.samcad"));
        assert_eq!(app.document_title(), Some("drawing".to_string()));

        // Path with no extension
        app.current_path = Some(PathBuf::from("/home/user/plan"));
        assert_eq!(app.document_title(), Some("plan".to_string()));

        // Hidden file
        app.current_path = Some(PathBuf::from("/home/user/.hidden.samcad"));
        assert_eq!(app.document_title(), Some(".hidden".to_string()));

        // Unicode filename
        app.current_path = Some(PathBuf::from("/home/user/図面.samcad"));
        assert_eq!(app.document_title(), Some("図面".to_string()));
    }

    #[test]
    fn clear_project_resets_drawing_and_camera() {
        let ctx = egui::Context::default();
        let tools = SharedTools::default();
        tools.select(Tool::Point);
        let mut app = default_test_app(ctx.clone(), tools);
        let mut harness = Harness::with_context(ctx);

        // Add a point
        harness.click(egui::pos2(100.0, 100.0), |ui| {
            app.ui(ui, &mut eframe::Frame::_new_kittest());
        });
        assert_eq!(points(&app.drawing).len(), 1);
        assert!(app.drawing.is_dirty());

        // Call clear_project directly (internal method, test via ui)
        // We can't call it directly since it's private, but we can test new_project
        // which calls clear_project when not dirty
        app.clear_project();

        assert_eq!(points(&app.drawing).len(), 0);
        assert!(!app.drawing.is_dirty());
        assert_eq!(app.camera, Camera::default());
        assert_eq!(app.current_path, None);
    }

    #[test]
    fn new_project_clears_immediately_when_not_dirty() {
        let ctx = egui::Context::default();
        let tools = SharedTools::default();
        let mut app = default_test_app(ctx.clone(), tools);
        let _harness = Harness::with_context(ctx);

        // Ensure drawing is clean
        assert!(!app.drawing.is_dirty());

        // Call new_project - should clear immediately without dialog
        app.new_project();

        // Drawing should be cleared
        assert_eq!(points(&app.drawing).len(), 0);
        assert!(!app.drawing.is_dirty());
        assert_eq!(app.current_path, None);
    }

    #[test]
    fn new_project_opens_dialog_when_dirty() {
        let ctx = egui::Context::default();
        let tools = SharedTools::default();
        tools.select(Tool::Point);
        let mut app = default_test_app(ctx.clone(), tools);
        let mut harness = Harness::with_context(ctx);

        // Add a point to make drawing dirty
        harness.click(egui::pos2(100.0, 100.0), |ui| {
            app.ui(ui, &mut eframe::Frame::_new_kittest());
        });
        assert!(app.drawing.is_dirty());

        // Call new_project - should open dialog
        app.new_project();

        // Dialog should be open
        assert!(app.new_project_dialog.is_open());
    }

    #[test]
    fn key_handlers_do_not_panic_on_shortcuts() {
        let ctx = egui::Context::default();
        let tools = SharedTools::default();
        let mut app = default_test_app(ctx.clone(), tools);
        let mut harness = Harness::with_context(ctx);

        // Test all keyboard shortcuts don't panic
        // (Zenity dialogs may open but won't block in headless mode)

        // Ctrl+S: Save
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
                    key: egui::Key::S,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: Default::default(),
                },
            ],
            |ui| app.ui(ui, &mut eframe::Frame::_new_kittest()),
        );

        // Ctrl+Shift+S: Save As
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
                    key: egui::Key::S,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: Default::default(),
                },
            ],
            |ui| app.ui(ui, &mut eframe::Frame::_new_kittest()),
        );

        // Ctrl+O: Open
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
                    key: egui::Key::O,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: Default::default(),
                },
            ],
            |ui| app.ui(ui, &mut eframe::Frame::_new_kittest()),
        );

        // All shortcuts handled without panic
    }

    #[test]
    fn escape_clears_the_active_tool() {
        let ctx = egui::Context::default();
        let tools = SharedTools::default();
        tools.select(Tool::Line);
        let mut app = default_test_app(ctx.clone(), tools.clone());
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
        let mut app = default_test_app(ctx.clone(), tools);
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
        let mut app = default_test_app(ctx.clone(), tools);
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
        let mut app = default_test_app(ctx.clone(), tools);
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

    #[test]
    fn save_as_uses_file_dialog_and_saves() {
        let ctx = egui::Context::default();
        let tools = SharedTools::default();
        let mut mock_dialog = MockFileDialog::default();
        mock_dialog.save_dialog_result = Some(PathBuf::from("/tmp/test.samcad"));
        let mut app = test_app(ctx.clone(), tools.clone(), mock_dialog);
        let mut harness = Harness::with_context(ctx);

        // Add a point to make drawing dirty
        tools.select(Tool::Point);
        harness.click(egui::pos2(100.0, 100.0), |ui| {
            app.ui(ui, &mut eframe::Frame::_new_kittest());
        });
        assert!(app.drawing.is_dirty());

        // Trigger Save As
        app.save_as();

        // Should have saved (path set, dirty cleared)
        assert_eq!(app.current_path, Some(PathBuf::from("/tmp/test.samcad")));
        assert!(!app.drawing.is_dirty());
    }

    #[test]
    fn save_as_returns_false_when_user_cancels() {
        let ctx = egui::Context::default();
        let tools = SharedTools::default();
        let mock_dialog = MockFileDialog::default();
        // Default is None (user cancelled)
        let mut app = test_app(ctx.clone(), tools, mock_dialog);

        let result = app.save_as();

        assert!(!result);
        assert_eq!(app.current_path, None);
    }

    #[test]
    fn save_as_new_uses_unique_path() {
        let ctx = egui::Context::default();
        let tools = SharedTools::default();
        let mut mock_dialog = MockFileDialog::default();
        mock_dialog.save_new_dialog_result = Some(PathBuf::from("/tmp/test (2).samcad"));
        let mut app = test_app(ctx.clone(), tools.clone(), mock_dialog);
        let mut harness = Harness::with_context(ctx);

        tools.select(Tool::Point);
        harness.click(egui::pos2(100.0, 100.0), |ui| {
            app.ui(ui, &mut eframe::Frame::_new_kittest());
        });

        let result = app.save_as_new();

        assert!(result);
        assert_eq!(
            app.current_path,
            Some(PathBuf::from("/tmp/test (2).samcad"))
        );
        assert!(!app.drawing.is_dirty());
    }

    #[test]
    fn save_without_path_triggers_save_as() {
        let ctx = egui::Context::default();
        let tools = SharedTools::default();
        let mut mock_dialog = MockFileDialog::default();
        mock_dialog.save_dialog_result = Some(PathBuf::from("/tmp/saved.samcad"));
        let mut app = test_app(ctx.clone(), tools.clone(), mock_dialog);
        let mut harness = Harness::with_context(ctx);

        tools.select(Tool::Point);
        harness.click(egui::pos2(100.0, 100.0), |ui| {
            app.ui(ui, &mut eframe::Frame::_new_kittest());
        });
        assert_eq!(app.current_path, None);

        let result = app.save();

        assert!(result);
        assert_eq!(app.current_path, Some(PathBuf::from("/tmp/saved.samcad")));
        assert!(!app.drawing.is_dirty());
    }

    #[test]
    fn save_with_existing_path_overwrites_silently() {
        let ctx = egui::Context::default();
        let tools = SharedTools::default();
        let mut app = default_test_app(ctx.clone(), tools.clone());
        let mut harness = Harness::with_context(ctx);

        tools.select(Tool::Point);
        harness.click(egui::pos2(100.0, 100.0), |ui| {
            app.ui(ui, &mut eframe::Frame::_new_kittest());
        });
        app.current_path = Some(PathBuf::from("/tmp/existing.samcad"));

        let result = app.save();

        assert!(result);
        assert_eq!(
            app.current_path,
            Some(PathBuf::from("/tmp/existing.samcad"))
        );
        assert!(!app.drawing.is_dirty());
    }

    #[test]
    fn open_without_dirty_changes_loads_immediately() {
        let ctx = egui::Context::default();
        let tools = SharedTools::default();
        let mut mock_dialog = MockFileDialog::default();
        mock_dialog.open_dialog_result = Some(PathBuf::from("/tmp/project.samcad"));
        let mut app = test_app(ctx.clone(), tools, mock_dialog);
        let _harness = Harness::with_context(ctx);

        // Drawing is clean
        assert!(!app.drawing.is_dirty());

        // Trigger Open
        app.open();

        // Should have attempted to load (we can't verify load without file system)
        // but pending_open_path should be None and no dialog opened
        assert_eq!(app.pending_open_path, None);
        assert!(!app.open_project_dialog.is_open());
    }

    #[test]
    fn open_with_dirty_changes_opens_dialog() {
        let ctx = egui::Context::default();
        let tools = SharedTools::default();
        let mut mock_dialog = MockFileDialog::default();
        mock_dialog.open_dialog_result = Some(PathBuf::from("/tmp/project.samcad"));
        let mut app = test_app(ctx.clone(), tools.clone(), mock_dialog);
        let mut harness = Harness::with_context(ctx);

        // Make drawing dirty
        tools.select(Tool::Point);
        harness.click(egui::pos2(100.0, 100.0), |ui| {
            app.ui(ui, &mut eframe::Frame::_new_kittest());
        });
        assert!(app.drawing.is_dirty());

        // Trigger Open
        app.open();

        // Should have stored path and opened dialog
        assert_eq!(
            app.pending_open_path,
            Some(PathBuf::from("/tmp/project.samcad"))
        );
        assert!(app.open_project_dialog.is_open());
    }

    #[test]
    fn open_cancelled_when_user_cancels_dialog() {
        let ctx = egui::Context::default();
        let tools = SharedTools::default();
        let mock_dialog = MockFileDialog::default();
        // Default is None (cancelled)
        let mut app = test_app(ctx.clone(), tools, mock_dialog);

        app.open();

        assert_eq!(app.pending_open_path, None);
        assert!(!app.open_project_dialog.is_open());
    }
}
