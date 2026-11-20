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

## ADR-005 — Native documents are versioned JSON via serde

- **Context:** Need a save format that is simple, debuggable and stable across versions.
- **Decision:** `.samcad` = versioned JSON serialized with serde.
- **Consequences:** Human-readable and easy to migrate; larger files than binary formats. Revisit if large-document load times become a problem.

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

## Template for future ADRs

```markdown
## ADR-XXX — Short title
- **Context:** What problem are we solving?
- **Decision:** What did we choose?
- **Alternatives:** What else was considered?
- **Consequences:** What becomes easier/harder?
```
