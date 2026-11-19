//! Library root for `samcad`.
//!
//! The layers follow hexagonal architecture (ports and adapters):
//!
//! - [`core`] — the innermost layer: domain model and port trait definitions.
//!   It depends on no GUI framework and no I/O crate.
//! - [`application`] — use cases implementing the driving ports.
//! - [`presentation`] — UI/product decisions expressed as egui-agnostic
//!   components ready to be rendered.
//! - [`adapters`] — framework-specific code: the eframe UI renderer and the
//!   window-command bridge.
//!
//! Dependencies point inwards only:
//! `adapters` → `presentation` → `application` → `core`.
//! The binary in `src/main.rs` is the composition root that wires them together.

pub mod adapters;
pub mod application;
pub mod core;
pub mod presentation;
