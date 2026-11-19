use eframe::egui;

use crate::presentation::component::IconId;

/// egui-specific parametrization for components produced by presentation.
/// Sizes and assets live here, never in the presentation layer.
pub struct Theme {
    /// Height of the always-visible main bar (title + window controls).
    pub main_bar_height: f32,
    /// Height of the dynamic bar that appears just below it (File menu).
    pub top_bar_height: f32,
    pub panel_width: f32,
    pub button_size: egui::Vec2,
    pub icon_size: egui::Vec2,
    /// Minimum size of the "File" menu button in the dynamic bar.
    pub menu_button_size: egui::Vec2,
    /// Minimum size of entries inside an open menu (e.g. "New Project").
    pub menu_item_size: egui::Vec2,
    /// Font size used for the File button and its menu entries.
    pub menu_font_size: f32,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            main_bar_height: 32.0,
            top_bar_height: 40.0,
            panel_width: 64.0,
            button_size: egui::vec2(48.0, 48.0),
            icon_size: egui::vec2(28.0, 28.0),
            menu_button_size: egui::vec2(80.0, 32.0),
            menu_item_size: egui::vec2(160.0, 32.0),
            menu_font_size: 16.0,
        }
    }
}

impl Theme {
    /// Maps a logical icon to the egui image source for its SVG asset.
    pub fn icon_source(&self, icon: IconId) -> egui::ImageSource<'static> {
        match icon {
            IconId::Point => egui::include_image!("icons/point.svg"),
            IconId::Line => egui::include_image!("icons/line.svg"),
            IconId::Circle => egui::include_image!("icons/circle.svg"),
            IconId::Rectangle => egui::include_image!("icons/rectangle.svg"),
            IconId::Arc => egui::include_image!("icons/arc.svg"),
        }
    }
}
