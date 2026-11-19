//! Presentation logic for the permanent main bar.
//!
//! Unlike the dynamic bar below it, this bar is always displayed. It holds the
//! version title and the window controls. No egui dependency here.

use crate::presentation::component::WindowButton;

/// The cooked contents of the permanent main bar.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MainBarModel {
    pub title: String,
    pub window_buttons: Vec<WindowButton>,
}

/// Build the main bar view model.
pub fn view_model() -> MainBarModel {
    MainBarModel {
        title: format!("SamCad v{}", env!("CARGO_PKG_VERSION")),
        window_buttons: vec![WindowButton::Minimize, WindowButton::Close],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn view_model_contains_title_and_window_controls() {
        let model = view_model();

        assert!(model.title.starts_with("SamCad v"));
        assert_eq!(
            model.window_buttons,
            vec![WindowButton::Minimize, WindowButton::Close]
        );
    }
}
