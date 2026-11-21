# STORY-002: Load native `.samcad` file

> Status: **Done** · Priority: **P0** · Related: FR-02, FR-03, FR-09e

## Requirement

Load a drawing from a `.samcad` file via Ctrl+O or File → Open Project, restoring both geometry and camera view. Opening with unsaved changes first asks **Save** / **Don't Save** / **Cancel** (FR-09e); a clean drawing loads immediately.

## Acceptance Criteria

- [x] Pressing **Ctrl+O** or clicking **File → Open Project** opens a native file dialog filtered to `.samcad`.
- [x] On selecting a valid `.samcad` file:
  - File is parsed as plain JSON matching the v1 schema.
  - `samcad_version` is checked; v1 loads, an unknown version returns `UnsupportedVersion`.
  - All points are restored to the document with exact world coordinates.
  - Camera `origin` and `zoom` are restored; view matches saved state.
  - If camera data is missing (old file), `Camera::from_data()` falls back to `Camera::default()` (1:1, origin 0,0).
  - The loaded drawing replaces the current document in a fresh session.
- [x] Invalid/malformed JSON returns a `Deserialize` error, does not crash, and leaves the current drawing untouched.
- [x] Missing file returns an `Io` error.
- [x] A legacy base64-encrypted file is detected and returns `LegacyEncryptedFormat` with a migration message instead of a generic parse failure.
- [x] With unsaved changes, the Open File confirmation dialog is shown; each choice follows the save scenarios in `../docs/functional-requirements.md`.
- [x] `cargo fmt`, `cargo check`, `cargo build`, `cargo test` pass.

## Technical Notes

- `LoadUseCase` + `LoadController` in `src/application/load.rs` return `LoadResult { document, camera }`.
- `SamcadApp::load_file()` in `adapters/ui/app.rs` replaces the document and camera, sets the current path and marks the drawing clean.
- Reuses the `FileSystem` driven port pattern from STORY-001 (read side, same module).
- File dialog is the mockable `FileDialog` port with `ZenityFileDialog` implementation (ADR-009).

## Dependencies

- STORY-001 (core format, FileSystem, dialog, ADR-009).

## Exit Criteria

Manual test: save from STORY-001 → Ctrl+O → select file → points + camera identical. Automated load tests cover restore, portability, missing file, malformed JSON, unsupported version and legacy encrypted content.