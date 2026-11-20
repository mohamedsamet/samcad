# Roadmap

> Status: **Draft** · Last updated: 2026-11-20
> Estimates are deliberately coarse; each phase ends with a runnable, demonstrable increment.

## Phase 0 — Research & foundations ✅

**Goal:** Validate the stack before writing product code.

- [x] Create `requirements/docs/` and write requirements.
- [x] Research GUI, DXF and PDF stacks.
- [x] Spike 1: egui/eframe window with a pannable/zoomable canvas drawing 10k lines.
- [ ] Spike 2: `dxf` crate round-trip a small drawing (line/polyline/circle/arc/text + layers).
- [ ] Spike 3: `printpdf` export a small vector drawing with text.
- [x] Generic modal dialog system with confirmation dialog for New Project.
- [ ] Record spikes as ADRs in `decisions.md`; adjust targets in `non-functional-requirements.md`.

**Exit criteria:** All three spikes run; measured startup/pan/zoom numbers recorded; stack confirmed or revised.

## Phase 1 — MVP editor (P0)

**Goal:** Draw and edit a basic 2D layout.

- Workspace crates: `core`, `io`, `app`, `cli` skeleton.
- Document model, layers, view transform, command history/undo.
- Tools: line, polyline, rectangle, circle, arc, text, point.
- Selection (click + window), move, delete, property editing (basic).
- Grid, pan/zoom/zoom-to-fit, coordinate readout, snapping (grid/endpoint), ortho.
- Native `.samcad` save/load, autosave recovery.
- Keyboard shortcuts: Ctrl+Z (undo), Ctrl+Y and Ctrl+Shift+Z (redo), Escape (clear tool).

**Exit criteria:** A user can draw, edit, undo and save/reopen a layout; `cargo test` green; no crashes on normal use.

## Phase 2 — Interchange & output (P1)

**Goal:** Get data in and out reliably.

- DXF import/export for the supported subset with import report for skipped entities.
- PDF export with paper size + scale options.
- Keyboard shortcuts/cheat sheet, command hint line, theme.
- Golden tests for DXF/PDF; validate output with an external tool.

**Exit criteria:** Round-trip a representative drawing: `.samcad → .dxf → reopen in LibreCAD/QCAD` and `.pdf` prints correctly with vector geometry and text.

## Phase 3 — Precision & productivity (P1/P2)

**Goal:** Make it pleasant for real drafting.

- Full snap modes (midpoint, center, intersection), polar/ortho refinements.
- Copy/offset, trim/extend, fillet, rotate/scale/mirror/array.
- Per-entity style overrides, dimensions/leaders, hatches, blocks/inserts.
- SVG export; headless CLI conversion for batch jobs.

**Exit criteria:** A modest architectural floor plan can be produced entirely in samcad with dimensions and exported to DXF + PDF.

## Phase 4 — Platform polish (P2)

**Goal:** Reach more users and sharpen performance.

- WASM build and browser file open/save.
- Named views, multiple viewports, plotting/title blocks.
- Performance pass: culling, batching, optional lyon/femtovg renderer if needed.
- Localization-ready strings, accessibility improvements.

**Exit criteria:** Web demo usable; performance targets from `non-functional-requirements.md` measured and met or explicitly revised.

## Phase 5 — Extended compatibility (P2+)

**Goal:** Handle formats and workflows beyond the MVP.

- Read-only DWG support via `libredwg` (if demand and licensing justify it).
- Newer DXF entity types, dimension styles, custom linetypes.
- Plugin/scripting evaluation (only after the core API is stable).

**Exit criteria:** Decision documented in a new ADR before starting.

## Cross-cutting definition of done

- Requirements docs updated in the same change.
- Unit tests added for new geometry/commands; golden test for new file formats/entities.
- `cargo fmt`, `cargo clippy -D warnings`, `cargo test` all pass.
- Manual smoke test: draw → save → reopen → export → open exported file elsewhere.
