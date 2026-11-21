//! Global adapter configuration loaded from `config/samcad.env`.
//!
//! The file is read once at startup from the `config/` directory relative to
//! the working directory (like Spring Boot's `application.properties`). It holds
//! graphics-related parameters today (theme colours, snap indicator, etc.) and
//! will grow to include user preferences.

use eframe::egui;
use std::fs;
use std::path::PathBuf;

use crate::adapters::util::env_parser::{parse_env_file, parse_rgba};

/// All graphics-related settings loaded from `config/samcad.env`.
#[derive(Debug, Clone)]
pub struct GraphicsConfig {
    /// Colour of the sticky-snap indicator circle.
    pub snap_indicator_color: egui::Color32,

    /// Colour of selected/highlighted UI elements (buttons, etc.).
    pub selection_color: egui::Color32,

    /// Point marker colour.
    pub point_color: egui::Color32,

    /// Point marker radius in pixels.
    pub point_radius: f32,
}

impl Default for GraphicsConfig {
    fn default() -> Self {
        Self {
            snap_indicator_color: egui::Color32::from_rgb(255, 165, 0), // orange
            selection_color: egui::Color32::from_rgb(0, 120, 215),      // blue
            point_color: egui::Color32::WHITE,
            point_radius: 2.0,
        }
    }
}

/// Loads the full adapter configuration.
///
/// Looks for `config/samcad.env` relative to the current working directory.
/// Missing or malformed file falls back to defaults.
pub fn load_config() -> AdapterConfig {
    let config_dir = std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("config");

    let env_path = config_dir.join("samcad.env");
    let raw = fs::read_to_string(&env_path).unwrap_or_default();
    let map = parse_env_file(&raw);

    let graphics = GraphicsConfig {
        snap_indicator_color: map
            .get("SNAP_INDICATOR_COLOR")
            .and_then(|v| parse_rgba(v))
            .unwrap_or(egui::Color32::from_rgb(255, 165, 0)),
        selection_color: map
            .get("SELECTION_COLOR")
            .and_then(|v| parse_rgba(v))
            .unwrap_or(egui::Color32::from_rgb(0, 120, 215)),
        point_color: map
            .get("POINT_COLOR")
            .and_then(|v| parse_rgba(v))
            .unwrap_or(egui::Color32::WHITE),
        point_radius: map
            .get("POINT_RADIUS")
            .and_then(|v| v.parse().ok())
            .unwrap_or(2.0),
    };

    AdapterConfig { graphics }
}

/// Top-level adapter configuration (graphics today, more sections later).
#[derive(Debug, Clone, Default)]
pub struct AdapterConfig {
    pub graphics: GraphicsConfig,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // Guard working-directory changes so parallel tests don't race.
    static CWD_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn missing_config_dir_uses_defaults() {
        let _g = CWD_LOCK.lock().unwrap();
        let tmp = std::env::temp_dir().join("samcad-test-cfg-missing");
        fs::create_dir_all(&tmp).ok();
        let old = std::env::current_dir().unwrap();
        std::env::set_current_dir(&tmp).unwrap();
        let cfg = load_config();
        std::env::set_current_dir(old).unwrap();
        fs::remove_dir_all(&tmp).ok();

        assert_eq!(cfg.graphics.point_radius, 2.0);
        assert_eq!(cfg.graphics.point_color, egui::Color32::WHITE);
        assert_eq!(
            cfg.graphics.snap_indicator_color,
            egui::Color32::from_rgb(255, 165, 0)
        );
        assert_eq!(
            cfg.graphics.selection_color,
            egui::Color32::from_rgb(0, 120, 215)
        );
    }

    #[test]
    fn config_file_overrides_defaults() {
        let _g = CWD_LOCK.lock().unwrap();
        let tmp = std::env::temp_dir().join("samcad-test-cfg-file");
        fs::create_dir_all(tmp.join("config")).unwrap();
        fs::write(
            tmp.join("config/samcad.env"),
            "SNAP_INDICATOR_COLOR=0,255,0\nSELECTION_COLOR=255,0,0\nPOINT_RADIUS=3.5\n",
        )
        .unwrap();
        let old = std::env::current_dir().unwrap();
        std::env::set_current_dir(&tmp).unwrap();
        let cfg = load_config();
        std::env::set_current_dir(old).unwrap();
        fs::remove_dir_all(&tmp).ok();

        assert_eq!(
            cfg.graphics.snap_indicator_color,
            egui::Color32::from_rgb(0, 255, 0)
        );
        assert_eq!(
            cfg.graphics.selection_color,
            egui::Color32::from_rgb(255, 0, 0)
        );
        assert_eq!(cfg.graphics.point_radius, 3.5);
    }

    #[test]
    fn malformed_values_fall_back_to_defaults() {
        let _g = CWD_LOCK.lock().unwrap();
        let tmp = std::env::temp_dir().join("samcad-test-cfg-bad");
        fs::create_dir_all(tmp.join("config")).unwrap();
        fs::write(
            tmp.join("config/samcad.env"),
            "POINT_RADIUS=not-a-number\nSNAP_INDICATOR_COLOR=999,0,0\n",
        )
        .unwrap();
        let old = std::env::current_dir().unwrap();
        std::env::set_current_dir(&tmp).unwrap();
        let cfg = load_config();
        std::env::set_current_dir(old).unwrap();
        fs::remove_dir_all(&tmp).ok();

        assert_eq!(cfg.graphics.point_radius, 2.0); // default
        assert_eq!(
            cfg.graphics.snap_indicator_color,
            egui::Color32::from_rgb(255, 165, 0)
        ); // default
    }

    #[test]
    fn graphics_config_default_values() {
        let cfg = GraphicsConfig::default();
        assert_eq!(cfg.point_radius, 2.0);
        assert_eq!(cfg.point_color, egui::Color32::WHITE);
        assert_eq!(
            cfg.snap_indicator_color,
            egui::Color32::from_rgb(255, 165, 0)
        );
        assert_eq!(cfg.selection_color, egui::Color32::from_rgb(0, 120, 215));
    }

    #[test]
    fn adapter_config_default() {
        let cfg = AdapterConfig::default();
        assert_eq!(cfg.graphics.point_radius, 2.0);
        assert_eq!(cfg.graphics.point_color, egui::Color32::WHITE);
    }

    #[test]
    fn load_config_with_partial_env_file() {
        let _g = CWD_LOCK.lock().unwrap();
        let tmp = std::env::temp_dir().join("samcad-test-cfg-partial");
        fs::create_dir_all(tmp.join("config")).unwrap();
        fs::write(tmp.join("config/samcad.env"), "POINT_RADIUS=5.0\n").unwrap();
        let old = std::env::current_dir().unwrap();
        std::env::set_current_dir(&tmp).unwrap();
        let cfg = load_config();
        std::env::set_current_dir(old).unwrap();
        fs::remove_dir_all(&tmp).ok();

        assert_eq!(cfg.graphics.point_radius, 5.0);
        // Other values should be defaults
        assert_eq!(
            cfg.graphics.snap_indicator_color,
            egui::Color32::from_rgb(255, 165, 0)
        );
        assert_eq!(
            cfg.graphics.selection_color,
            egui::Color32::from_rgb(0, 120, 215)
        );
        assert_eq!(cfg.graphics.point_color, egui::Color32::WHITE);
    }

    #[test]
    fn load_config_with_four_component_rgba() {
        let _g = CWD_LOCK.lock().unwrap();
        let tmp = std::env::temp_dir().join("samcad-test-cfg-rgba");
        fs::create_dir_all(tmp.join("config")).unwrap();
        fs::write(
            tmp.join("config/samcad.env"),
            "SNAP_INDICATOR_COLOR=255,0,0,128\n",
        )
        .unwrap();
        let old = std::env::current_dir().unwrap();
        std::env::set_current_dir(&tmp).unwrap();
        let cfg = load_config();
        std::env::set_current_dir(old).unwrap();
        fs::remove_dir_all(&tmp).ok();

        assert_eq!(
            cfg.graphics.snap_indicator_color,
            egui::Color32::from_rgba_unmultiplied(255, 0, 0, 128)
        );
    }
}
