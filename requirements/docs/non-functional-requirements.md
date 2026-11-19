# Non-Functional Requirements

> Status: **Draft** · Last updated: 2026-11-19
> Targets marked *(to benchmark)* are hypotheses to validate with a small spike before committing.

## Performance

| ID | Requirement | Target |
| --- | --- | --- |
| NFR-01 | Cold startup | < 1 s on a mid-range laptop *(to benchmark)* |
| NFR-02 | Idle CPU | ~0% when no interaction/animation (egui repaint-on-demand) |
| NFR-03 | Interactive pan/zoom | 60 fps with 10k visible entities; still usable at 50k *(to benchmark)* |
| NFR-04 | DXF import | 50k entities parsed in < 2 s on a mid-range laptop *(to benchmark)* |
| NFR-05 | PDF export | Typical A3 drawing exported in < 2 s |
| NFR-06 | Memory | < 200 MB for a 50k-entity drawing *(to benchmark)* |

Techniques: world→screen transform with view-frustum culling, cheap entity bounds, batch painting by layer/style, avoid per-frame allocations, defer heavy work out of the UI thread via channels.

## Simplicity & deployment

- Single binary with no runtime interpreter, no external services, no mandatory network access.
- Installer is a single executable/Archive per platform; no system-wide dependencies beyond the OS window stack.
- Default configuration works out of the box; settings stored in a small user config file.
- The entire MVP codebase should stay reviewable by one developer; prefer standard-library solutions before adding dependencies.

## Portability

- Linux, macOS and Windows from the same codebase via `eframe`.
- Core and I/O crates must not depend on GUI or OS-specific code.
- Keep a WASM-compatible path: avoid blocking I/O and non-WASM crates in the core and app layers when practical.

## Reliability & correctness

- Malformed or unsupported DXF must never crash the app; report skipped entities in an import report.
- Never lose user work silently: autosave recovery file at a configurable interval; atomic writes (write temp + rename).
- Undo/redo must be deterministic and cover every mutating command.
- Geometric predicates use explicit epsilon tolerances; no `NaN` coordinates stored in documents.
- Exported DXF/PDF must be validated against at least one independent tool in CI (LibreCAD/QCAD CLI or a PDF parser).

## Compatibility

- DXF: read/write ASCII R12 and R2000 subsets in v1; newer versions added incrementally.
- PDF: PDF 1.7-compatible vector output, selectable paper sizes and scale.
- Native documents: versioned schema (`version` field) so future changes can migrate old files.

## Maintainability

- UI-independent `samcad-core` crate with unit tests for geometry and commands.
- Golden-file tests for DXF/PDF exporters.
- Public API of core/io documented with rustdoc; every ADR in `decisions.md` records a non-obvious choice.
- `cargo fmt`, `cargo clippy -D warnings` and `cargo test` green on every commit.

## Security & privacy

- Offline by default; no telemetry, no auto-update checks without explicit opt-in.
- No execution of scripts/macros from drawing files.
- File parsing must be bounded (no unbounded reads/allocations from untrusted DXF).

## Licensing

- Prefer MIT / Apache-2.0 / BSD dependencies so samcad can be redistributed or commercialized later.
- Avoid GPL/AGPL dependencies in the core stack. (`dxf`, `egui`, `printpdf`, `lyon` are all MIT and/or Apache-2.0.)
- Bundle only fonts with redistributable licenses (e.g. OFL).

## Accessibility

- Keyboard-driven operation for all drawing tools (important for CAD workflows).
- egui's AccessKit support enabled where available; high-contrast theme considered in later phases.
