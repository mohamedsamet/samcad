# STORY-003: Save As and native file dialogs

> Status: **Planned** · Priority: **P0** · Related: FR-03, FR-09a

## Requirement

Support Save As (Ctrl+Shift+S) and ensure all file operations use native dialogs.

## Acceptance Criteria

- [ ] **Ctrl+Shift+S** or **File → Save As** always opens a native save dialog, even if file already has a path.
- [ ] Save As updates the current file path for subsequent Ctrl+S saves.
- [ ] **Ctrl+O** / **File → Open** uses native open dialog filtered to `.samcad`.
- [ ] Dialogs are centered, modal, and respect platform conventions (Linux/Windows/macOS).
- [ ] Overwrite confirmation when saving to an existing file (native dialog handles this).
- [ ] `cargo fmt`, `cargo build`, `cargo test` pass.

## Technical Notes

- Extend `SaveUseCase` with `save_as(path)` separate from `save()`.
- `SaveController` tracks `current_path: Option<PathBuf>`.
- File dialog logic shared in `src/adapters/ui/file_dialog.rs`:
  - `save_dialog(default_name: &str) -> Option<PathBuf>`
  - `open_dialog() -> Option<PathBuf>`
- Uses `rfd::FileDialog` with filter `("SamCAD files", &["samcad"])`.

## Dependencies

- STORY-001, STORY-002.

## Exit Criteria

Manual test: new drawing → Ctrl+Shift+S → name it → Ctrl+S (no dialog) → modify → Ctrl+S → reopen → all changes present.