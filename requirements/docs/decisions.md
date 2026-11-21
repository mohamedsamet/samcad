# Architecture Decision Records (ADRs)

> Status: **Draft** · Last updated: 2026-11-21

## ADR-001 — Use Rust as the implementation language

- **Context:** Need a fast, safe, portable desktop app with precise numeric geometry and a single-binary distribution.
- **Decision:** Rust (stable toolchain).
- **Consequences:** Excellent performance/safety; steeper learning curve and slower UI iteration than scripting languages. Mitigated by egui's simple API.

## ADR-002 — Use egui/eframe for the UI

- **Context:** Need a simple, fast GUI with a custom drawing canvas, cross-platform and ideally WASM-capable.
- **Decision:** `eframe` + `egui`, custom painting via `egui::Painter`/`epaint`.
- **Consequences:** Very little UI boilerplate, repaint-on-demand, native + web from one codebase. Accept that the widget look is not native and that egui APIs may break between versions — pin versions. Revisit with a custom wgpu renderer only if benchmarks require it.

## ADR-003 — DXF is the primary CAD interchange; no DWG write in v1

- **Context:** Users expect "CAD files". DWG is proprietary and has no simple, license-friendly Rust writer. DXF is open, widely supported and sufficient for 2D handoff.
- **Decision:** Use the `dxf` crate for DXF read/write (ASCII R12/R2000 subset first). Do not implement DWG writing. Consider read-only DWG via `libredwg` later.
- **Consequences:** Interoperability with AutoCAD/LibreCAD/QCAD via DXF; must clearly document the limitation. Simpler, safer implementation.

## ADR-004 — Use printpdf for vector PDF export

- **Context:** PDF output must be printable/vector, not a screenshot.
- **Decision:** `printpdf`, drawing entity geometry as PDF path operators with typed units; bundle one OFL-licensed font initially.
- **Consequences:** Clean scalable output and selectable paper sizes; font embedding/shaping needs careful early testing.

## ADR-005 — Native documents are plain versioned JSON via serde (no encryption)

- **Context:** Need a save format that is simple, debuggable and stable across versions. An earlier build encrypted files with a keyring-backed AES-GCM scheme, which made files unreadable, added dependencies, and required secret storage.
- **Decision:** `.samcad` = plain, human-readable versioned JSON serialized with serde. No encryption, keyring or secret storage is involved. Legacy base64/encrypted files are detected on load and rejected with `LegacyEncryptedFormat` and a migration message.
- **Alternatives:** Keep AES-GCM + keyring encryption; use a binary format (`postcard`) for compactness.
- **Consequences:** Files are portable, inspectable and easy to migrate; larger than binary formats and carry no confidentiality. Revisit only if a concrete confidentiality requirement appears.

## ADR-006 — Keep a UI-independent core crate

- **Context:** CAD geometry/commands must be testable headlessly and reusable by a CLI and potential web build.
- **Decision:** `samcad-core` has no UI dependencies; all mutation goes through commands; `io` and `app` depend on it.
- **Consequences:** Slightly more boilerplate; much better testability, clean undo/redo and a straightforward headless CLI.

## ADR-007 — lyon for path/tessellation math, not a full geometry kernel

- **Context:** Need curves, filled shapes and a path abstraction without pulling in a C++ CAD kernel.
- **Decision:** Use `lyon` for path building/curve math/tessellation when needed; no OpenCASCADE or equivalent.
- **Consequences:** Lightweight and pure Rust; advanced booleans/offsets may require `geo` or custom code later.

## ADR-008 — Generic modal dialog system for confirmations

- **Context:** "New Project" needs to ask whether to save current work before discarding it. Future flows (discard changes, overwrite file, etc.) will need the same pattern.
- **Decision:** A reusable `DialogModel<A>` / `DialogButton<A>` / `DialogState` trio in `presentation/dialog.rs`, rendered by a generic `dialog::show()` adapter in `adapters/ui/dialog.rs`. Buttons are right-aligned; Escape and backdrop click close without acting.
- **Alternatives:** Hard-code a one-off modal; use egui's `Window` instead of `Modal`.
- **Consequences:** New confirmations only need a view model function and a `DialogState` field. Slight extra abstraction for the first use, but pays off immediately for the next one.

## ADR-009 — File dialogs behind a mockable `FileDialog` port using zenity

- **Context:** Save, Save As and Open need native file dialogs. Calling `zenity` directly from `SamcadApp` made headless tests hang for over 60 seconds waiting for a dialog that never closes, and left `file_dialog.rs` at ~36% coverage.
- **Decision:** Define a `FileDialog` trait (`save_dialog`, `save_new_dialog`, `open_dialog`) in `adapters/ui/file_dialog.rs`. `ZenityFileDialog` invokes `zenity --file-selection` in production; `MockFileDialog` returns configurable paths or cancellations in tests. `SamcadApp` is generic over `F: FileDialog` and the concrete implementation is injected from the composition root.
- **Alternatives:** Add an `rfd` / native dialog crate (extra dependency, still hard to test); keep free functions and skip dialog tests; set an environment variable to bypass dialogs.
- **Consequences:** Dialog flows are unit-tested without spawning subprocesses; adding another platform implementation means one more trait impl. `unique_path` and extension enforcement stay in the adapter and remain directly testable.

## ADR-010 — Dirty tracking with save confirmation on New Project and Open

- **Context:** New Project and Open must not silently discard unsaved work, but must also stay quick when there is nothing to lose.
- **Decision:** `DrawingUseCase` exposes `is_dirty()` / `mark_saved()`. New Project resets immediately when clean and otherwise shows a **Save / Don't Save** dialog; Open behaves the same with an extra **Cancel** choice. Save persists the current path silently, while Save As always opens a dialog and generates a unique name. The main bar renders the document stem with a trailing `*` while dirty.
- **Alternatives:** Always confirm regardless of dirty state; autosave before every destructive action; use a single generic "are you sure" prompt with no save option.
- **Consequences:** No interruption for clean drawings, no data loss for dirty ones, and a single dirty flag drives both the title indicator and the confirmation flow. The dialog view models (`presentation/new_project.rs`, `presentation/open_project.rs`) hold the product decisions and remain egui-agnostic.

## Template for future ADRs

```markdown
## ADR-XXX — Short title
- **Context:** What problem are we solving?
- **Decision:** What did we choose?
- **Alternatives:** What else was considered?
- **Consequences:** What becomes easier/harder?
```
