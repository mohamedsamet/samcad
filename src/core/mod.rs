//! Innermost layer: domain model and port definitions.
//!
//! Nothing in here may depend on egui/eframe, the filesystem, or any other
//! external system. Domain entities (document, geometry, commands) will live
//! alongside the ports as the feature set grows.

pub mod command;
pub mod crypto;
pub mod document;
pub mod format;
pub mod keystore;
pub mod machine;
pub mod model;
pub mod ports;
