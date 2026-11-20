//! Native `.samcad` file format — versioned JSON via serde.
//!
//! This module contains the serializable data structures for the native file
//! format. It has no I/O dependencies and lives in `core` so it can be used
//! by both the application layer and headless tools.

use serde::{Deserialize, Serialize};

/// Top-level file format. Versioned so we can migrate later.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SamcadFile {
    /// File format version. Increment on breaking schema changes.
    pub samcad_version: u32,

    /// When the file was saved (ISO-8601 UTC).
    pub saved_at: String,

    /// Machine that created the file.
    pub machine: MachineInfo,

    /// Drawing document data.
    pub document: DocumentData,

    /// Camera view state (zoom/pan) for exact view restore.
    pub camera: CameraData,
}

impl SamcadFile {
    /// Current file format version.
    pub const CURRENT_VERSION: u32 = 1;
}

/// Machine identity captured at save time.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MachineInfo {
    /// System hostname.
    pub hostname: String,

    /// Primary non-loopback IPv4 address, or "unknown".
    pub ip: String,
}

/// Serializable document data.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DocumentData {
    /// Document schema version (separate from file format version).
    pub version: u32,

    /// Points in world coordinates (f64, Y-up).
    pub points: Vec<PointData>,
}

impl DocumentData {
    pub const CURRENT_VERSION: u32 = 1;
}

/// Serializable point (world coordinates, Y-up).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct PointData {
    pub x: f64,
    pub y: f64,
}

/// Serializable camera state.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct CameraData {
    /// World coordinate at canvas top-left (origin.x, origin.y).
    pub origin_x: f64,
    pub origin_y: f64,

    /// Screen points per world unit.
    pub zoom: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn samcad_file_round_trip() {
        let file = SamcadFile {
            samcad_version: SamcadFile::CURRENT_VERSION,
            saved_at: "2026-11-20T15:30:00Z".to_string(),
            machine: MachineInfo {
                hostname: "test-host".to_string(),
                ip: "192.168.1.42".to_string(),
            },
            document: DocumentData {
                version: DocumentData::CURRENT_VERSION,
                points: vec![
                    PointData { x: 12.5, y: 34.0 },
                    PointData { x: -3.5, y: 4.25 },
                ],
            },
            camera: CameraData {
                origin_x: 0.0,
                origin_y: 0.0,
                zoom: 1.0,
            },
        };

        let json = serde_json::to_string_pretty(&file).unwrap();
        let parsed: SamcadFile = serde_json::from_str(&json).unwrap();
        assert_eq!(file, parsed);
    }

    #[test]
    fn point_data_serializes_as_object() {
        let pt = PointData { x: 1.5, y: -2.5 };
        let json = serde_json::to_string(&pt).unwrap();
        assert_eq!(json, r#"{"x":1.5,"y":-2.5}"#);
    }

    #[test]
    fn camera_data_serializes_correctly() {
        let cam = CameraData {
            origin_x: 100.0,
            origin_y: -50.0,
            zoom: 2.5,
        };
        let json = serde_json::to_string(&cam).unwrap();
        let parsed: CameraData = serde_json::from_str(&json).unwrap();
        assert_eq!(cam, parsed);
    }
}
