# STORY-004: Camera view persistence (zoom/pan)

> Status: **Done** · Priority: **P0** · Related: FR-02, FR-09d

## Requirement

Save and restore camera zoom and pan in the `.samcad` file so the view is identical on reopen.

## Acceptance Criteria

- [x] Camera `origin` (world coordinate at canvas top-left) is saved as `camera.origin_x`, `camera.origin_y` (f64).
- [x] Camera `zoom` (screen points per world unit) is saved as `camera.zoom` (f32, clamped 0.01–10000).
- [x] On load, camera is reconstructed from saved values; view matches exactly.
- [x] `Camera::default()` (origin 0,0, zoom 1.0) is used for a fresh/cleared project; the v1 schema requires a `camera` object, so a missing one is a deserialize error rather than a silent fallback (there are no earlier JSON files to migrate — the previous format was encrypted).
- [x] If saved zoom is outside the clamp range, `Camera::from_data()` clamps it (defensive).
- [x] `cargo fmt`, `cargo check`, `cargo build`, `cargo test` pass.

## Technical Notes

- `to_data()` / `from_data()` on `Camera` in `src/presentation/canvas.rs` convert to/from `CameraData`.
- `CameraData` struct in `src/core/format.rs` mirrors the fields.
- `LoadController` returns `LoadResult { document, camera }`; `SamcadApp::load_file()` replaces `self.camera` on load.
- No UI changes needed — existing `paint_points` + `Camera::world_to_screen` works automatically.

## Dependencies

- STORY-001 (format structs), STORY-002 (load path).

## Exit Criteria

Manual test: zoom to 3.5x, pan to (100, -50) world, save → reopen → zoom reads 3.5x, pan at same world position. Covered by `application/load.rs` and `core/format.rs` unit tests.