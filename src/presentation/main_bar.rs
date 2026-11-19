//! Presentation logic for the permanent main bar.
//!
//! Unlike the dynamic bar below it, this bar is always displayed. It holds the
//! version title and the window controls. No egui dependency here.

use crate::presentation::component::WindowButton;

/// The cooked contents of the permanent main bar.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MainBarModel {
    pub title: &'static str,
    pub window_buttons: &'static [WindowButton],
}

/// Build the main bar view model. Everything is static, so painting the bar
/// every frame allocates nothing.
pub const fn view_model() -> MainBarModel {
    MainBarModel {
        title: concat!("SamCad v", env!("CARGO_PKG_VERSION")),
        window_buttons: &[WindowButton::Minimize, WindowButton::Close],
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
    }
}
