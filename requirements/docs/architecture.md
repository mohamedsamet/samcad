# Proposed Architecture

> Status: **Draft** · Last updated: 2026-11-19

## Crate layout

Start with a Cargo workspace so boundaries are enforced from day one; keep each crate small.

```text
samcad/
  Cargo.toml                  # workspace
  crates/
    samcad-core/              # document model, geometry, commands, units — NO UI deps
    samcad-io/                # DXF, PDF, SVG, native .samcad serialization (depends on core)
    samcad-app/               # egui/eframe UI, viewport, tools, input (depends on core + io)
    samcad-cli/               # clap-based headless conversion (depends on core + io)
  requirements/docs/          # these documents
```

Dependency direction is strictly one-way: `app`/`cli` → `io` → `core`. `core` depends only on math/serde-type crates.

If the very first spike needs to move faster, a single crate with `core/`, `io/` and `app/` modules is acceptable — split along the same boundaries as soon as the second feature lands.

## Implemented structure (hexagonal / ports and adapters)

The current single-crate layout applies hexagonal boundaries through modules. Dependencies point inwards only: `adapters` → `presentation` → `application` → `core`.

Because this is a UI-centric product, presentation is a first-class layer, not a disposable shell. It "cooks" egui-agnostic component descriptions; `adapters/ui` only holds egui-specific parametrization (sizes, SVG assets) and paints them.

```text
src/
├── main.rs                  # composition root: wiring only, no business logic
├── lib.rs                   # pub mod core / application / presentation / adapters
├── core/
│   ├── mod.rs               # domain entities + ports (no framework deps)
│   ├── model.rs             # Tool enum
│   └── ports.rs             # driving ports (WindowUseCase, ToolUseCase), driven port (WindowActions)
├── application/
│   ├── mod.rs
│   ├── window.rs            # WindowController implements WindowUseCase
│   └── tool.rs              # ToolController implements ToolUseCase
├── presentation/
│   ├── mod.rs
│   ├── component.rs         # Component / IconId / Emphasis — no egui types
│   └── palette.rs           # builds the palette component tree from tool state
└── adapters/
    ├── mod.rs
    ├── window.rs            # driven adapter (channel) + bridge to egui ViewportCommand
    └── ui/
        ├── mod.rs
        ├── app.rs           # driving adapter: SamcadApp + native_options()
        ├── theme.rs         # sizes + IconId → SVG asset mapping (egui-specific)
        ├── title_bar.rs     # top bar, version, minimize/close
        ├── tool_panel.rs    # renders presentation components with egui
        └── icons/           # line.svg, circle.svg, rectangle.svg, arc.svg
```

Flow for a click on **Close**: `title_bar` → `WindowUseCase::close()` → `WindowController` → `WindowActions::close()` → `WindowActionAdapter` queues a command → `WindowActionBridge` sends `ViewportCommand::Close` on the next frame.

Flow for the tool palette: `presentation::palette::components()` reads `ToolUseCase` and returns a `Vec<Component>` (label, separator, four `IconButton`s with tooltips and one selected) → `adapters/ui/tool_panel.rs` maps each component to an egui widget using `Theme` for sizes and `IconId → SVG` assets. A click calls `ToolUseCase::select()`. The component rules are unit-tested without egui. Canvas shape previews will be added as new `Component` variants produced by presentation and painted by the adapter.

When the project later splits into workspace crates, these modules map onto `samcad-core`, `samcad-presentation`, `samcad-app`, and a new `samcad-ports` crate (or ports stay in `core`).

## Core domain model (sketch)

```rust
// World coordinates are millimetres (f64). Screen coordinates are points (f32).
struct Point { x: f64, y: f64 }

struct Document {
    version: u32,
    units: Units,              // Mm default
    layers: Vec<Layer>,
    active_layer: LayerId,
    entities: Vec<Entity>,     // or slotmap for stable handles
}

struct Layer {
    id: LayerId,
    name: String,
    color: Color,
    line_weight: f64,
    visible: bool,
    locked: bool,
}

enum Entity {
    Line(Line),
    Polyline(Polyline),
    Rectangle(Rectangle),
    Circle(Circle),
    Arc(Arc),
    Text(Text),
}

struct EntityCommon {
    id: EntityId,
    layer: LayerId,
    color_override: Option<Color>,
    line_weight_override: Option<f64>,
}
```

- Prefer `f64` for world geometry and DXF fidelity; convert to `f32` only at paint time.
- Use stable entity handles (`EntityId`) so selection, undo and DXF handles map cleanly.
- Geometry types expose `bounds()`, `translate()`, `distance_to(point)` and `to_path()` (lyon) where relevant.

## Coordinate systems

| Space | Unit | Origin | Notes |
| --- | --- | --- | --- |
| World | mm (f64) | drawing origin | All stored geometry; Y up |
| Screen / egui | logical pt (f32) | top-left | Y down; egui convention |
| Paper (export) | mm/pt | page origin | PDF/DXF export transform |

`ViewTransform { pan: Vec2<f64>, zoom: f64, dpi }` converts world→screen with a Y flip. Zoom is around the cursor (keep the world point under the mouse fixed).

## Command & undo architecture

- Every mutation is a `Command` implementing `execute(&mut Document)` and `undo(&mut Document)` (or stores before/after snapshots for simplicity).
- `CommandHistory` owns undo/redo stacks; the UI only dispatches commands and never mutates `Document` directly.
- Drawing tools are small state machines: `Idle → AwaitingFirstPoint → AwaitingSecondPoint → … → Commit(Command)` with a live preview entity that is not in the document.
- This makes tools testable without a GUI and gives undo/redo for free.

## Rendering pipeline (MVP)

```text
Document + ViewTransform + Layer visibility
        │
        ├─ cull entities to visible bounds (cheap AABB test)
        ├─ group by layer/style to minimize state changes
        └─ egui::Painter: line / circle / polygon / text shapes + grid + snap markers
```

- egui repaints only on input/tool changes; no render loop needed when idle.
- Keep painting code behind a `CanvasRenderer` trait so we can swap in a lyon + `femtovg`/wgpu renderer later without rewriting tools.

## Export pipelines

```text
Document ──> samcad-io::dxf  ──> dxf::Drawing ──> .dxf
Document ──> samcad-io::pdf  ──> printpdf::PdfDocument ──> .pdf
Document ──> samcad-io::svg  ──> (later: svg2pdf / resvg)
Document ──> serde ──> versioned .samcad JSON
```

- One `Entity → DXF entity` and one `Entity → PDF ops` conversion per entity type, colocated in `samcad-io` so adding an entity type has a clear checklist.
- Export shares the same geometric primitives used by painting (bounds, path conversion) to avoid divergent behavior.

## File formats

- `.samcad`: versioned JSON (serde) — human-readable, easy to debug, good enough for MVP. Revisit with `ron`/`postcard` if size matters.
- `.dxf`: interchange format, ASCII subset, R12/R2000 first.
- `.pdf`: vector output, PDF 1.7.

## Error handling

- `core`/`io`: typed errors via `thiserror` (e.g. `DxfImportError`, `ExportError`).
- `app`/`cli`: `anyhow::Result` at boundaries, with user-friendly messages and import reports.
- Never `unwrap()` on file parsing paths.

## Testing strategy

- Unit tests: geometry math, snapping, transform round-trips, command undo/redo.
- Golden tests: export known documents to DXF/PDF and compare normalized output or parse-back structure.
- Property tests (optional): transform invariants, round-trip coordinates.
- Manual acceptance checklist per phase in `roadmap.md`.
