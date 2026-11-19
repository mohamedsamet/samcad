//! Presentation logic for the dynamic, auto-hiding bar that sits just below
//! the permanent main bar.
//!
//! Two things are cooked here, with no egui dependency:
//! 1. [`TopBarState`] — the reveal / hide state machine driven by cursor
//!    position, clicks and time.
//! 2. [`TopBarModel`] — the view model of what the bar contains (only File).

use crate::presentation::component::{MenuAction, MenuItem};

/// How long the bar stays visible after the cursor leaves it, in seconds.
pub const HIDE_DELAY: f64 = 10.0;

/// State of the auto-hide behaviour. The egui adapter feeds it cursor and
/// clock input each frame and paints the bar only while `update` returns true.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TopBarState {
    visible: bool,
    hide_at: Option<f64>,
}

impl TopBarState {
    /// Hide immediately when the user clicks below the bar's height.
    ///
    /// The bar spans the window width, so only the Y coordinate matters.
    /// Clicks on the bar itself (or no click this frame) are ignored.
    pub fn notify_click(&mut self, click_y: Option<f32>, bar_bottom: f32) {
        if click_y.is_some_and(|y| y > bar_bottom) {
            self.visible = false;
            self.hide_at = None;
        }
    }

    /// Advance the state machine.
    ///
    /// - `cursor_y`: distance from the top of the window, `None` when the
    ///   cursor is not over the window.
    /// - `bar_bottom`: Y coordinate of the bottom edge of the dynamic bar.
    ///   Reaching this height reveals the bar; leaving it starts the countdown.
    /// - `now`: current time in seconds (egui's `InputState::time`).
    ///
    /// Returns whether the bar should be painted this frame.
    pub fn update(&mut self, cursor_y: Option<f32>, bar_bottom: f32, now: f64) -> bool {
        let cursor_reached_bar = cursor_y.is_some_and(|y| (0.0..=bar_bottom).contains(&y));

        if cursor_reached_bar {
            // Cursor reached the bar's height (or is still on one of the bars).
            self.visible = true;
            self.hide_at = None;
        } else if self.visible {
            // Cursor left the bar's height: start/continue the hide countdown.
            let deadline = *self.hide_at.get_or_insert(now + HIDE_DELAY);
            if now >= deadline {
                self.visible = false;
                self.hide_at = None;
            }
        }

        self.visible
    }

    /// Whether the bar is currently visible (does not advance the state).
    pub fn is_visible(&self) -> bool {
        self.visible
    }
}

/// A top-level menu, e.g. "File".
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Menu {
    pub label: &'static str,
    pub items: Vec<MenuItem>,
}

/// The cooked contents of the dynamic bar: menus only. The version title and
/// window controls belong to the permanent main bar.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TopBarModel {
    pub menus: Vec<Menu>,
}

/// Build the dynamic bar view model. All product decisions live here; the egui
/// adapter only maps these values onto widgets.
pub fn view_model() -> TopBarModel {
    TopBarModel {
        menus: vec![Menu {
            label: "File",
            items: vec![MenuItem {
                label: "New Project",
                action: MenuAction::NewProject,
            }],
        }],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Main bar (32) + dynamic bar (40).
    const BAR_BOTTOM: f32 = 72.0;

    #[test]
    fn hidden_until_the_cursor_reaches_the_bars_height() {
        let mut state = TopBarState::default();
        assert!(!state.is_visible());

        // Still hidden while the cursor is below the bar's height.
        assert!(!state.update(Some(80.0), BAR_BOTTOM, 0.0));

        // Reaching the bar's height reveals it.
        assert!(state.update(Some(72.0), BAR_BOTTOM, 0.0));
    }

    #[test]
    fn stays_visible_while_cursor_is_on_either_bar() {
        let mut state = TopBarState::default();
        state.update(Some(10.0), BAR_BOTTOM, 0.0);

        for time in 1..10 {
            assert!(state.update(Some(60.0), BAR_BOTTOM, time as f64));
        }
        assert!(state.is_visible());
    }

    #[test]
    fn hides_ten_seconds_after_cursor_leaves_the_bars() {
        let mut state = TopBarState::default();
        state.update(Some(10.0), BAR_BOTTOM, 0.0);

        assert!(state.update(Some(80.0), BAR_BOTTOM, 1.0));
        assert!(state.update(Some(80.0), BAR_BOTTOM, 10.9));
        assert!(!state.update(Some(80.0), BAR_BOTTOM, 11.0));
    }

    #[test]
    fn countdown_restarts_when_cursor_returns_then_leaves_again() {
        let mut state = TopBarState::default();
        state.update(Some(10.0), BAR_BOTTOM, 0.0);

        assert!(state.update(Some(80.0), BAR_BOTTOM, 3.0));
        assert!(state.update(Some(10.0), BAR_BOTTOM, 4.0)); // resets timer
        assert!(state.update(Some(80.0), BAR_BOTTOM, 5.0));
        assert!(state.update(Some(80.0), BAR_BOTTOM, 14.9));
        assert!(!state.update(Some(80.0), BAR_BOTTOM, 15.0));
    }

    #[test]
    fn cursor_outside_window_starts_the_countdown() {
        let mut state = TopBarState::default();
        state.update(Some(10.0), BAR_BOTTOM, 0.0);

        assert!(state.update(None, BAR_BOTTOM, 1.0));
        assert!(!state.update(None, BAR_BOTTOM, 11.0));
    }

    #[test]
    fn click_outside_hides_immediately() {
        let mut state = TopBarState::default();
        assert!(state.update(Some(10.0), BAR_BOTTOM, 0.0));
        assert!(state.is_visible());

        state.notify_click(Some(200.0), BAR_BOTTOM);
        assert!(!state.is_visible());
    }

    #[test]
    fn click_inside_the_bar_does_not_hide_it() {
        let mut state = TopBarState::default();
        assert!(state.update(Some(10.0), BAR_BOTTOM, 0.0));

        state.notify_click(Some(50.0), BAR_BOTTOM);
        assert!(state.is_visible());
    }

    #[test]
    fn view_model_contains_only_the_file_menu() {
        let model = view_model();

        assert_eq!(model.menus.len(), 1);
        assert_eq!(model.menus[0].label, "File");
        assert_eq!(model.menus[0].items.len(), 1);
        assert_eq!(model.menus[0].items[0].label, "New Project");
        assert_eq!(model.menus[0].items[0].action, MenuAction::NewProject);
    }
}
