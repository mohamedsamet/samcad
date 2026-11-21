use eframe::egui;

use crate::core::ports::WindowUseCase;
use crate::presentation::component::{MenuAction, WindowButton};
use crate::presentation::main_bar;

use super::theme::Theme;

/// Renders the permanent main bar: File menu on the left, document title after it,
/// app title in the middle, window controls on the right.
/// Returns the menu action selected this frame, if any.
pub fn show(
    ui: &mut egui::Ui,
    window: &dyn WindowUseCase,
    theme: &Theme,
    model: &main_bar::MainBarModel,
) -> Option<MenuAction> {
    let response = egui::Panel::top("main_bar")
        .exact_size(theme.main_bar_height)
        .show(ui, |ui| {
            ui.columns(3, |columns| {
                // Left column: File menu + document title
                let chosen = columns[0].vertical_centered(|ui| {
                    ui.horizontal(|ui| {
                        let menu_result = paint_menus(ui, model.menus, theme);
                        if let Some(doc_title) = &model.document_title {
                            ui.add_space(8.0);
                            ui.label(doc_title.as_str());
                        }
                        menu_result
                    })
                    .inner
                });

                // Center column: app title (version)
                columns[1].vertical_centered(|ui| {
                    ui.label(model.app_title.as_str());
                });

                // Right column: window controls
                columns[2].with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    for &button in model.window_buttons.iter().rev() {
                        paint_window_button(ui, button, window);
                    }
                });

                chosen
            })
            .inner
        });
    response.inner
}

fn paint_menus(ui: &mut egui::Ui, menus: &[main_bar::Menu], theme: &Theme) -> Option<MenuAction> {
    let mut chosen = None;

    ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
        // Reduce vertical button padding so the menu button fits exactly
        // within the main bar height without overflowing.
        let original_padding = ui.style().spacing.button_padding;
        ui.style_mut().spacing.button_padding = egui::vec2(8.0, 0.0);

        for menu in menus {
            let label = egui::RichText::new(menu.label).size(theme.menu_font_size);
            let button = egui::Button::new(label).min_size(theme.menu_button_size);
            egui::menu::MenuButton::from_button(button).ui(ui, |ui| {
                for item in menu.items {
                    let item_label = egui::RichText::new(item.label).size(theme.menu_font_size);
                    let item_button = egui::Button::new(item_label).min_size(theme.menu_item_size);
                    if ui.add(item_button).clicked() {
                        // The caller decides what the action does; the menu
                        // only records the selection and closes itself.
                        chosen = Some(item.action);
                        ui.close();
                    }
                }
            });
        }

        ui.style_mut().spacing.button_padding = original_padding;
    });

    chosen
}

fn paint_window_button(ui: &mut egui::Ui, button: WindowButton, window: &dyn WindowUseCase) {
    let (glyph, tooltip) = match button {
        WindowButton::Minimize => ("−", "Minimize"),
        WindowButton::Close => ("×", "Close"),
    };

    if ui.button(glyph).on_hover_text(tooltip).clicked() {
        match button {
            WindowButton::Minimize => window.minimize(),
            WindowButton::Close => window.close(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::adapters::ui::test_support::{Harness, RecordingWindow};
    use crate::presentation::component::MenuItem;

    #[test]
    fn render_keeps_the_bar_free_of_window_commands_until_clicked() {
        let mut harness = Harness::new();
        let window = RecordingWindow::default();

        let model = main_bar::view_model(None, false);
        harness.frame(vec![], |ui| {
            show(ui, &window, &Theme::default(), &model);
        });

        assert!(window.calls.borrow().is_empty());
    }

    #[test]
    fn both_window_buttons_call_their_window_port_method() {
        let mut harness = Harness::new();
        let window = RecordingWindow::default();
        let button_pos = egui::pos2(10.0, 10.0);

        for button in [WindowButton::Minimize, WindowButton::Close] {
            harness.click(button_pos, |ui| {
                paint_window_button(ui, button, &window);
            });
        }

        assert_eq!(*window.calls.borrow(), ["minimize", "close"]);
    }

    #[test]
    fn show_renders_without_panic() {
        let mut harness = Harness::new();
        let window = RecordingWindow::default();

        let model = main_bar::view_model(None, false);
        harness.frame(vec![], |ui| {
            show(ui, &window, &Theme::default(), &model);
        });
        // If we get here without panic, the test passes
    }

    #[test]
    fn paint_menus_with_empty_list_returns_none() {
        let mut harness = Harness::new();

        harness.frame(vec![], |ui| {
            let result = paint_menus(ui, &[], &Theme::default());
            assert!(result.is_none());
        });
    }

    #[test]
    fn paint_menus_with_menu_items_renders() {
        let mut harness = Harness::new();

        let menu = main_bar::Menu {
            label: "File",
            items: &[
                MenuItem {
                    label: "New",
                    action: MenuAction::NewProject,
                },
                MenuItem {
                    label: "Open",
                    action: MenuAction::Open,
                },
            ],
        };

        harness.frame(vec![], |ui| {
            let result = paint_menus(ui, &[menu], &Theme::default());
            assert!(result.is_none());
        });
    }
}
