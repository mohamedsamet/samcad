# STORY-001: Save native `.samcad` file

> Status: **Done** · Priority: **P0** · Related: FR-02, FR-03, FR-09a, FR-09d

## Requirement

Save the current drawing to a native `.samcad` file (plain versioned JSON, no encryption) via Ctrl+S or File → Save. Save overwrites a known path silently; the first save opens a native dialog. After a successful save the drawing is marked clean and the main bar shows the document stem without the dirty `*`.

## Acceptance Criteria

- [x] Pressing **Ctrl+S** or clicking **File → Save** opens a native save dialog (if no path yet) or overwrites the current file without a dialog.
- [x] File is written as valid, human-readable JSON matching the v1 schema in `PLAN-samcad-save-format.md`.
- [x] File contains:
  - `samcad_version: 1`
  - `saved_at` (ISO-8601 UTC, via `time::OffsetDateTime`)
  - `machine` { `hostname`, `ip` }
  - `document` { `version: 1`, `points: [...]` }
  - `camera` { `origin_x`, `origin_y`, `zoom` }
- [x] All points in the current document are saved with their world coordinates (f64, Y-up).
- [x] Current camera zoom and pan are saved; view is restored on load (STORY-004).
- [x] After save, the main bar shows the file stem and clears the dirty `*` (FR-09d).
- [x] No encryption, keyring or secret storage is involved; output is portable plain JSON.
- [x] `cargo fmt`, `cargo check`, `cargo build`, `cargo test` pass.

## Technical Notes

- `src/core/format.rs` holds `SamcadFile`, `DocumentData`, `PointData`, `CameraData`, `MachineInfo` serde structs.
- `Document::to_data()` / `Document::from_data()` in `core/document.rs`.
- `Camera::to_data()` / `Camera::from_data()` in `presentation/canvas.rs`.
- `SaveUseCase` + `SaveController` in `src/application/save.rs`; `FileSystem` driven port and `StdFileSystem` live in the same module.
- File dialog is the mockable `FileDialog` port with `ZenityFileDialog` implementation in `src/adapters/ui/file_dialog.rs` (ADR-009).
- Ctrl+S / Ctrl+Shift+S / Ctrl+O wired in `adapters/ui/app.rs`; the app owns the current path and calls `SaveController::save_as()`.

## Dependencies

- `serde`, `serde_json` in `Cargo.toml`.
- `time` for the ISO-8601 timestamp; `gethostname` and `local-ip-address` for machine info.
- No native dialog crate: `zenity` is invoked via `std::process::Command`.

## Exit Criteria

Manual test: draw 3 points, zoom 2x, pan, Ctrl+S → reopen → points + camera identical. Automated round-trip and plain-JSON content tests live in `application/save.rs`, `application/load.rs` and `core/format.rs`.