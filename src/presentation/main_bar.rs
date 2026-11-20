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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MainBarModel {
    pub title: &'static str,
    pub window_buttons: &'static [WindowButton],
    pub menus: &'static [Menu],
}

/// Build the main bar view model. Everything is static, so painting the bar
/// every frame allocates nothing.
pub const fn view_model() -> MainBarModel {
    MainBarModel {
        title: concat!("SamCad v", env!("CARGO_PKG_VERSION")),
        window_buttons: &[WindowButton::Minimize, WindowButton::Close],
        menus: &[Menu {
            label: "File",
            items: &[
                MenuItem {
                    label: "New Project",
                    action: crate::presentation::component::MenuAction::NewProject,
                },
                MenuItem {
                    label: "Open",
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
        let model = view_model();

        assert_eq!(
            model.title,
            format!("SamCad v{}", env!("CARGO_PKG_VERSION"))
        );
        assert_eq!(
            model.window_buttons,
            [WindowButton::Minimize, WindowButton::Close].as_slice()
        );
        assert_eq!(model.menus.len(), 1);
        assert_eq!(model.menus[0].label, "File");
    }
}
