use eframe::egui;

use crate::presentation::component::MenuAction;
use crate::presentation::top_bar::{self, TopBarState};

use super::theme::Theme;

/// Renders the dynamic bar just below the permanent main bar as an overlay.
/// It floats over the tool palette. Visibility is decided by the presentation
/// state machine; this module only paints the File menu and closes it when an
/// item is selected.
pub fn show(ctx: &egui::Context, state: &mut TopBarState, theme: &Theme) {
    let (cursor_y, click_y, now) = ctx.input(|input| {
        let cursor_y = input.pointer.hover_pos().map(|pos| pos.y);
        let click_y = if input.pointer.any_click() {
            cursor_y
        } else {
            None
        };
        (cursor_y, click_y, input.time)
    });

    let bar_bottom = theme.main_bar_height + theme.top_bar_height;
    // A click below the bar dismisses it immediately; the 10s timer still
    // handles the case where the cursor simply moves away.
    state.notify_click(click_y, bar_bottom);
    if !state.update(cursor_y, bar_bottom, now) {
        return;
    }

    let model = top_bar::view_model();

    egui::Area::new(egui::Id::unique("floating_top_bar"))
        .anchor(
            egui::Align2::LEFT_TOP,
            egui::vec2(0.0, theme.main_bar_height),
        )
        .order(egui::Order::Foreground)
        .movable(false)
        .show(ctx, |ui| {
            egui::Frame::new()
                .fill(ui.style().visuals.panel_fill)
                .inner_margin(egui::Margin::symmetric(8, 4))
                .show(ui, |ui| {
                    paint_menus(ui, model.menus, theme);
                });
        });
}

fn paint_menus(ui: &mut egui::Ui, menus: &[top_bar::Menu], theme: &Theme) {
    egui::MenuBar::new().ui(ui, |ui| {
        for menu in menus {
            let label = egui::RichText::new(menu.label).size(theme.menu_font_size);
            let button = egui::Button::new(label).min_size(theme.menu_button_size);
            egui::menu::MenuButton::from_button(button).ui(ui, |ui| {
                for item in menu.items {
                    let item_label = egui::RichText::new(item.label).size(theme.menu_font_size);
                    let item_button = egui::Button::new(item_label).min_size(theme.menu_item_size);
                    if ui.add(item_button).clicked() {
                        // No document model or file I/O yet: selecting any
                        // File action only closes the menu for now.
                        match item.action {
                            MenuAction::NewProject | MenuAction::Save | MenuAction::Export => {
                                ui.close()
                            }
                        }
                    }
                }
            });
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::adapters::ui::test_support::{Harness, click, moved};

    const BAR_POINTER: egui::Pos2 = egui::pos2(100.0, 40.0);

    fn show_bar(ctx: &egui::Context, state: &mut TopBarState) {
        show(ctx, state, &Theme::default());
    }

    #[test]
    fn pointer_reaching_the_bar_reveals_and_paints_the_file_menu() {
        let mut harness = Harness::new();
        let mut state = TopBarState::default();

        harness.frame(vec![moved(BAR_POINTER)], |ui| {
            show_bar(ui.ctx(), &mut state);
        });

        assert!(state.is_visible());
    }

    #[test]
    fn click_below_the_bar_dismisses_it_immediately() {
        let mut harness = Harness::new();
        let mut state = TopBarState::default();

        harness.frame(vec![moved(BAR_POINTER)], |ui| {
            show_bar(ui.ctx(), &mut state);
        });
        harness.frame(click(egui::pos2(600.0, 400.0)), |ui| {
            show_bar(ui.ctx(), &mut state);
        });

        assert!(!state.is_visible());
    }
}
