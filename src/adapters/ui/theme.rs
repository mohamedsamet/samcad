use eframe::egui;

use crate::presentation::component::IconId;

/// egui-specific parametrization for components produced by presentation.
/// Sizes and assets live here, never in the presentation layer.
pub struct Theme {
    pub panel_width: f32,
    pub button_size: egui::Vec2,
    pub icon_size: egui::Vec2,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            panel_width: 56.0,
            button_size: egui::vec2(40.0, 40.0),
            icon_size: egui::vec2(24.0, 24.0),
        }
    }
}

impl Theme {
    /// Maps a logical icon to the egui image source for its SVG asset.
    pub fn icon_source(&self, icon: IconId) -> egui::ImageSource<'static> {
        match icon {
            IconId::Line => egui::include_image!("icons/line.svg"),
            IconId::Circle => egui::include_image!("icons/circle.svg"),
            IconId::Rectangle => egui::include_image!("icons/rectangle.svg"),
            IconId::Arc => egui::include_image!("icons/arc.svg"),
        }
    }
}
