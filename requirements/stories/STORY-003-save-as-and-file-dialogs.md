# STORY-003: Save As and native file dialogs

> Status: **Done** · Priority: **P0** · Related: FR-03, FR-09a, FR-09f

## Requirement

Support Save As (Ctrl+Shift+S / File → Save As...) and ensure all file operations use native dialogs through a mockable port. Save As always opens a dialog and never overwrites an existing file: a numeric suffix is added instead.

## Acceptance Criteria

- [x] **Ctrl+Shift+S** or **File → Save As...** always opens a native save dialog, even if the file already has a path.
- [x] Save As updates the current file path for subsequent Ctrl+S saves (which then overwrite silently).
- [x] If the selected `.samcad` path already exists, a numeric suffix ` (2)`, ` (3)`, … is inserted before the extension (FR-09f); hidden files and empty extensions are handled.
- [x] The `.samcad` extension is enforced on every path returned by a dialog (added or replaced).
- [x] **Ctrl+O** / **File → Open Project** uses the native open dialog filtered to `.samcad`.
- [x] The dialog port is injectable: `MockFileDialog` returns canned paths/cancellations in unit tests, so headless runs never spawn a native window.
- [x] `cargo fmt`, `cargo check`, `cargo build`, `cargo test` pass.

## Technical Notes

- `SaveUseCase::save_as(path, drawing, camera)` in `application/save.rs` is separate from `save()`; the app owns the current path.
- `src/adapters/ui/file_dialog.rs` exposes the `FileDialog` trait (ADR-009) with:
  - `save_dialog(&str) -> Option<PathBuf>` (first save, native overwrite confirmation),
  - `save_new_dialog(&str) -> Option<PathBuf>` (Save As, applies `unique_path`),
  - `open_dialog() -> Option<PathBuf>`.
- `ZenityFileDialog` implements the trait on Linux using `zenity --file-selection` with `--save` and `--confirm-overwrite` where appropriate; no `rfd` dependency and no blocking window in tests.
- `MockFileDialog` (`#[cfg(test)]`) implements the same trait with configurable results; `SamcadApp` is generic over `F: FileDialog`.

## Dependencies

- STORY-001, STORY-002, ADR-009 (file dialog port).

## Exit Criteria

Manual test: new drawing → Ctrl+Shift+S → name it → Ctrl+S (no dialog) → modify → Ctrl+S → reopen → all changes present. Automated tests cover unique-name generation and each Save/Open scenario through the mock dialog.