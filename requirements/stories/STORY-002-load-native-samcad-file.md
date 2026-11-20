# STORY-002: Load native `.samcad` file

> Status: **Planned** · Priority: **P0** · Related: FR-02, FR-03

## Requirement

Load a drawing from a `.samcad` file via Ctrl+O or File → Open, restoring both geometry and camera view.

## Acceptance Criteria

- [ ] Pressing **Ctrl+O** or clicking **File → Open** opens a native file dialog filtered to `.samcad`.
- [ ] On selecting a valid `.samcad` file:
  - File is parsed as JSON matching the v1 schema.
  - `samcad_version` is checked; v1 loads, unknown version shows error dialog.
  - All points are restored to the document with exact world coordinates.
  - Camera `origin` and `zoom` are restored; view matches saved state.
  - If camera data is missing (old file), default to `Camera::default()` (1:1, origin 0,0).
  - Undo/redo history is cleared (fresh session).
- [ ] Invalid/malformed JSON shows error dialog, does not crash.
- [ ] `cargo fmt`, `cargo build`, `cargo test` pass.

## Technical Notes

- `LoadUseCase` + `LoadController` in `src/application/load.rs`.
- Returns `(DocumentData, CameraData)` to caller.
- `WindowController` / `SamcadApp` replaces document + camera.
- Reuses `FileSystem` driven port from STORY-001.
- File dialog via `rfd` (same as save).

## Dependencies

- STORY-001 (core format, FileSystem, dialog).

## Exit Criteria

Manual test: save from STORY-001 → Ctrl+O → select file → points + camera identical.