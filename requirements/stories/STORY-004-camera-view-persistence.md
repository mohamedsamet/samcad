# STORY-004: Camera view persistence (zoom/pan)

> Status: **Planned** · Priority: **P0** · Related: FR-02 (implied)

## Requirement

Save and restore camera zoom and pan in the `.samcad` file so the view is identical on reopen.

## Acceptance Criteria

- [ ] Camera `origin` (world coordinate at canvas top-left) is saved as `camera.origin_x`, `camera.origin_y` (f64).
- [ ] Camera `zoom` (screen points per world unit) is saved as `camera.zoom` (f32, clamped 0.01–10000).
- [ ] On load, camera is reconstructed from saved values; view matches exactly.
- [ ] If `camera` object is missing from file (backwards compat), default to `Camera::default()` (origin 0,0, zoom 1.0).
- [ ] If saved zoom is outside clamp range, clamp on load (defensive).
- [ ] `cargo fmt`, `cargo build`, `cargo test` pass.

## Technical Notes

- Add `to_data()` / `from_data()` to `Camera` in `src/presentation/canvas.rs`.
- `CameraData` struct in `src/core/format.rs` mirrors the fields.
- `LoadController` returns `CameraData` alongside `DocumentData`.
- `SamcadApp` (or `WindowController`) replaces `self.camera` on load.
- No UI changes needed — existing `paint_points` + `Camera::world_to_screen` works automatically.

## Dependencies

- STORY-001 (format structs), STORY-002 (load path).

## Exit Criteria

Manual test: zoom to 3.5x, pan to (100, -50) world, save → reopen → zoom reads 3.5x, pan at same world position.