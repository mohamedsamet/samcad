use eframe::egui;

use crate::core::ports::ToolUseCase;
use crate::presentation::component::{Component, Emphasis};
use crate::presentation::palette;

use super::theme::Theme;

/// Thin egui renderer: asks presentation for the palette components, then
/// paints them using the adapter's theme. It makes no product decisions.
pub fn show(ui: &mut egui::Ui, tools: &dyn ToolUseCase, theme: &Theme) {
    egui::Panel::left("tool_panel")
        .exact_size(theme.panel_width)
        .show(ui, |ui| {
            ui.vertical_centered(|ui| {
                for component in palette::components(tools) {
                    paint(ui, component, theme, tools);
                }
            });
        });
}

fn paint(ui: &mut egui::Ui, component: Component, theme: &Theme, tools: &dyn ToolUseCase) {
    match component {
        Component::Label { text, emphasis } => match emphasis {
            Emphasis::Strong => {
                ui.label(egui::RichText::new(text).small().strong());
            }
            Emphasis::Normal => {
                ui.label(text);
            }
        },
        Component::Separator => {
            ui.separator();
        }
        Component::IconButton {
            tool,
            icon,
            tooltip,
            selected,
            enabled,
        } => {
            let image = egui::Image::new(theme.icon_source(icon))
                .fit_to_exact_size(theme.icon_size)
                .alt_text(tooltip);

            let button = egui::Button::new(image)
                .min_size(theme.button_size)
                .selected(selected)
                .image_tint_follows_text_color(true);

            let response = ui.add_enabled(enabled, button);
            if response.clicked() {
                tools.select(tool);
            }
            response.on_hover_text(tooltip);
        }
    }
}
