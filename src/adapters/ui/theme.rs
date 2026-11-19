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
    /// Radius of a drawn point marker, in points.
    pub point_radius: f32,
    /// Colour of a drawn point marker.
    pub point_color: egui::Color32,
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
            point_radius: 2.0,
            point_color: egui::Color32::WHITE,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_theme_keeps_the_expected_metrics() {
        let theme = Theme::default();

        assert_eq!(theme.main_bar_height, 32.0);
        assert_eq!(theme.top_bar_height, 40.0);
        assert_eq!(theme.panel_width, 64.0);
        assert_eq!(theme.menu_font_size, 16.0);
        assert_eq!(theme.point_radius, 2.0);
        assert_eq!(theme.point_color, egui::Color32::WHITE);
    }

    #[test]
    fn every_icon_maps_to_its_embedded_svg() {
        let theme = Theme::default();
        let icons = [
            (IconId::Point, "bytes://icons/point.svg"),
            (IconId::Line, "bytes://icons/line.svg"),
            (IconId::Circle, "bytes://icons/circle.svg"),
            (IconId::Rectangle, "bytes://icons/rectangle.svg"),
            (IconId::Arc, "bytes://icons/arc.svg"),
        ];

        for (icon, expected) in icons {
            match theme.icon_source(icon) {
                egui::ImageSource::Bytes { uri, .. } => {
                    assert_eq!(uri.as_ref(), expected);
                }
                _ => panic!("expected an embedded SVG for {icon:?}"),
            }
        }
    }
}
