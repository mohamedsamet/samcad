//! Presentation logic for the permanent main bar.
//!
//! Unlike the dynamic bar below it, this bar is always displayed. It holds the
//! version title and the window controls. No egui dependency here.

use crate::presentation::component::{MenuItem, WindowButton};

/// A top-level menu, e.g. "File".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Menu {
    pub label: &'static str,
    pub items: &'static [MenuItem],
}

/// The cooked contents of the permanent main bar.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MainBarModel {
    /// App title shown in the center (e.g., "SamCad v0.1.0")
    pub app_title: String,
    /// Document title shown after the File menu (e.g., "drawing*" or "drawing")
    pub document_title: Option<String>,
    pub window_buttons: &'static [WindowButton],
    pub menus: &'static [Menu],
}

/// Build the main bar view model.
///
/// When a file is saved or opened, its name (without extension) is shown
/// after the File menu. An asterisk is appended while there are unsaved changes.
/// The app title (version) is always shown in the center.
pub fn view_model(document_title: Option<&str>, dirty: bool) -> MainBarModel {
    let document_title = document_title.map(|name| {
        if dirty {
            format!("{}*", name)
        } else {
            name.to_string()
        }
    });

    let app_title = format!("SamCad v{}", env!("CARGO_PKG_VERSION"));

    MainBarModel {
        app_title,
        document_title,
        window_buttons: &[WindowButton::Minimize, WindowButton::Close],
        menus: &[Menu {
            label: "File",
            items: &[
                MenuItem {
                    label: "New Project",
                    action: crate::presentation::component::MenuAction::NewProject,
                },
                MenuItem {
                    label: "Open Project",
                    action: crate::presentation::component::MenuAction::Open,
                },
                MenuItem {
                    label: "Save",
                    action: crate::presentation::component::MenuAction::Save,
                },
                MenuItem {
                    label: "Save As...",
                    action: crate::presentation::component::MenuAction::SaveAs,
                },
                MenuItem {
                    label: "Export",
                    action: crate::presentation::component::MenuAction::Export,
                },
            ],
        }],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn view_model_contains_title_and_window_controls() {
        let model = view_model(None, false);

        assert_eq!(
            model.app_title,
            format!("SamCad v{}", env!("CARGO_PKG_VERSION"))
        );
        assert_eq!(model.document_title, None);
        assert_eq!(
            view_model(Some("drawing"), false).document_title,
            Some("drawing".to_string())
        );
        assert_eq!(
            view_model(Some("drawing"), true).document_title,
            Some("drawing*".to_string())
        );
        assert_eq!(
            model.window_buttons,
            [WindowButton::Minimize, WindowButton::Close].as_slice()
        );
        assert_eq!(model.menus.len(), 1);
        assert_eq!(model.menus[0].label, "File");
    }
}
