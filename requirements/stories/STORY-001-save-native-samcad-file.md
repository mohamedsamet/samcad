# STORY-001: Save native `.samcad` file

> Status: **Planned** · Priority: **P0** · Related: FR-02, FR-09a

## Requirement

Save the current drawing to a native `.samcad` file (versioned JSON) via Ctrl+S or File → Save.

## Acceptance Criteria

- [ ] Pressing **Ctrl+S** or clicking **File → Save** opens a native save dialog (if no path yet) or overwrites the current file.
- [ ] File is written as valid JSON matching the v1 schema in `PLAN-samcad-save-format.md`.
- [ ] File contains:
  - `samcad_version: 1`
  - `saved_at` (ISO-8601 UTC)
  - `machine` { `hostname`, `ip` }
  - `document` { `version: 1`, `points: [...]` }
  - `camera` { `origin_x`, `origin_y`, `zoom` }
- [ ] All points in the current document are saved with their world coordinates (f64, Y-up).
- [ ] Current camera zoom and pan are saved.
- [ ] After save, the window title reflects the saved path (optional but nice).
- [ ] `cargo fmt`, `cargo build`, `cargo test` pass.

## Technical Notes

- New `src/core/format.rs` with serde structs.
- `Document::to_data()` / `Document::from_data()` in `document.rs`.
- `Camera::to_data()` / `Camera::from_data()` in `presentation/canvas.rs`.
- `SaveUseCase` + `SaveController` in `src/application/save.rs`.
- `FileSystem` driven port + `StdFileSystem` in `src/adapters/io/`.
- File dialog via `rfd` crate in `src/adapters/ui/file_dialog.rs`.
- Wire Ctrl+S in `adapters/ui/app.rs` → `SaveUseCase::save()`.

## Dependencies

- `serde`, `serde_json` in `Cargo.toml`.
- `gethostname`, `local-ip-address` for machine info.
- `rfd` for native file dialog.

## Exit Criteria

Manual test: draw 3 points, zoom 2x, pan, Ctrl+S → reopen → points + camera identical.