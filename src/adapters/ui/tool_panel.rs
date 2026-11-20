use eframe::egui;

use crate::core::ports::ToolUseCase;
use crate::presentation::component::Component;
use crate::presentation::palette;

use super::theme::Theme;

/// Thin egui renderer: asks presentation for the palette components, then
/// paints them using the adapter's theme. It makes no product decisions.
pub fn show(ui: &mut egui::Ui, tools: &dyn ToolUseCase, theme: &Theme) {
    egui::Panel::left("tool_panel")
        .exact_size(theme.panel_width)
        .show(ui, |ui| {
            ui.vertical_centered(|ui| {
                // No reserved space: the dynamic bar floats over this panel.
                for component in palette::components(tools.selected()) {
                    paint(ui, component, theme, tools);
                }
            });
        });
}

fn paint(ui: &mut egui::Ui, component: Component, theme: &Theme, tools: &dyn ToolUseCase) {
    match component {
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

#[cfg(test)]
mod tests {
    use super::*;

    use crate::adapters::ui::test_support::Harness;
    use crate::application::tool::ToolController;
    use crate::core::model::Tool;
    use crate::presentation::component::IconId;

    fn icon(tool: Tool, enabled: bool) -> Component {
        Component::IconButton {
            tool,
            icon: IconId::Circle,
            tooltip: "Circle",
            selected: false,
            enabled,
        }
    }

    #[test]
    fn render_paints_the_separator_and_every_palette_button() {
        let mut harness = Harness::new();
        let tools = ToolController::new();

        harness.frame(vec![], |ui| show(ui, &tools, &Theme::default()));

        assert_eq!(tools.selected(), None);
    }

    #[test]
    fn clicking_an_icon_selects_its_tool() {
        let mut harness = Harness::new();
        let tools = ToolController::new();

        harness.click(egui::pos2(20.0, 20.0), |ui| {
            paint(ui, icon(Tool::Circle, true), &Theme::default(), &tools);
        });

        assert_eq!(tools.selected(), Some(Tool::Circle));
    }

    #[test]
    fn disabled_icons_ignore_clicks() {
        let mut harness = Harness::new();
        let tools = ToolController::new();

        harness.click(egui::pos2(20.0, 20.0), |ui| {
            paint(ui, icon(Tool::Circle, false), &Theme::default(), &tools);
        });

        assert_eq!(tools.selected(), None);
    }
}
