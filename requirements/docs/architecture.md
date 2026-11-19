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

The current single-crate layout applies hexagonal boundaries through modules. Dependencies point inwards only: `application` and `presentation` depend on `core`; `adapters` depend on `core` and `presentation`; `main` wires application use cases into the adapters.

Because this is a UI-centric product, presentation is a first-class layer, not a disposable shell. It "cooks" egui-agnostic component descriptions; `adapters/ui` only holds egui-specific parametrization (sizes, SVG assets) and paints them.

```text
src/
├── main.rs                  # composition root: wiring only, no business logic
├── lib.rs                   # pub mod core / application / presentation / adapters
├── core/
│   ├── mod.rs               # domain entities + ports (no framework deps)
│   ├── model.rs             # Tool enum
│   ├── document.rs          # Point entity + Document container
│   └── ports.rs             # driving ports (WindowUseCase, ToolUseCase, DrawingUseCase), driven port (WindowActions)
├── application/
│   ├── mod.rs
│   ├── window.rs            # WindowController implements WindowUseCase
│   ├── tool.rs              # ToolController implements ToolUseCase
│   └── drawing.rs           # DrawingController implements DrawingUseCase
├── presentation/
│   ├── mod.rs
│   ├── canvas.rs            # camera (zoom/pan) + click action
│   ├── component.rs         # Component / IconId / MenuAction / MenuItem / WindowButton — no egui
│   ├── main_bar.rs          # permanent bar view model (title + window controls)
│   ├── palette.rs           # builds the palette component tree from tool state
│   └── top_bar.rs           # auto-hide state machine + dynamic bar view model (File only)
└── adapters/
    ├── mod.rs
    ├── window.rs            # driven adapter (channel) + bridge to egui ViewportCommand
    └── ui/
        ├── mod.rs
        ├── app.rs           # driving adapter: SamcadApp + native_options()
        ├── canvas.rs        # central drawing area + CAD cursor
        ├── theme.rs         # sizes + IconId → SVG asset mapping (egui-specific)
        ├── cursor.rs        # embedded bitmap crosshair cursor for the canvas
        ├── main_bar.rs      # permanent top Panel: title + window controls
        ├── top_bar.rs       # floating Area just below the main bar: File menu
        ├── tool_panel.rs    # renders presentation components with egui
        └── icons/           # point.svg, line.svg, circle.svg, rectangle.svg, arc.svg,
                             # cursor_crosshair.png/.rgba
```

Flow for a click on **Close**: `adapters/ui/main_bar.rs` → `WindowUseCase::close()` → `WindowController` → `WindowActions::close()` → `WindowActionAdapter` queues a command → `WindowActionBridge` sends `ViewportCommand::Close` on the next frame.

Flow for the tool palette: the adapter reads `tools.selected()` and passes it as plain data to `presentation::palette::components(selected)`, which returns an iterator over `Component`s (a separator and five `IconButton`s — Point, Line, Circle, Rectangle, Arc — with tooltips and one selected) → `adapters/ui/tool_panel.rs` maps each component to an egui widget using `Theme` for sizes and `IconId → SVG` assets. A click calls `ToolUseCase::select()`; pressing Esc in `adapters/ui/app.rs` calls `ToolUseCase::deselect()`. Presentation therefore depends only on `Tool`, not on the application ports, and the component rules are unit-tested without egui. Canvas shape previews will be added as new `Component` variants produced by presentation and painted by the adapter.

Two bars: `presentation::main_bar::view_model()` supplies the title and window controls to `adapters/ui/main_bar.rs`, which paints a permanent `Panel::top`. `presentation::top_bar` supplies the auto-hide state machine and the dynamic bar's menu-only view model; `adapters/ui/top_bar.rs` paints it as a foreground `egui::Area` anchored immediately below the main bar. Each frame the adapter feeds cursor Y and time into `TopBarState::update()` with the bottom edge of both bars as the reveal height: the dynamic bar appears when the cursor reaches that height, stays while the cursor is on either bar, and hides 10 s after the cursor leaves both; `TopBarState::notify_click()` dismisses it immediately on a click below that height. All of this is unit-tested without a window. The bar is a foreground `Area` that floats over the tool palette (no space is reserved for it there), and its File / New Project buttons use larger minimum sizes from `Theme`. Clicking **File → New Project / Save / Export** closes the menu via `ui.close()`; the actual operations are deferred (FR-01, FR-02, FR-05–FR-06). The tool palette has no "Tools" heading and uses larger 48×48 buttons (28×28 icons).

Flow for canvas input: `adapters/ui/canvas.rs` allocates the whole `CentralPanel` with `Sense::click_and_drag()`. On hover it reads `smooth_scroll_delta` for the wheel (and `zoom_delta` for Ctrl+wheel / trackpad pinch) and calls `presentation::canvas::Camera::zoom_around(cursor, factor)`, which keeps the world point under the cursor fixed and clamps the zoom to 0.01–10 000 px per world unit; middle-drag and primary-drag (only when no tool is active, so it never competes with click-to-draw) call `Camera::pan_by()`. A primary click converts the pointer position with `Camera::screen_to_world()` and asks `click_action(selected, world)`. With the Point tool it returns `AddPoint`; the adapter calls `DrawingUseCase::add_point()` on `DrawingController`, which appends to the `Document`. Painting calls `DrawingUseCase::for_each_point()` and `Camera::world_to_screen()`, rejecting points outside the viewport before drawing each white circle of `Theme::point_radius`. World coordinates are `f64` with Y up; canvas-local screen coordinates are `f32` with Y down, and the camera performs the flip. Geometry that predates a units/viewport transform is stored in canvas-independent world units, so it survives zooming and panning.

Canvas performance: egui is reactive, so an idle canvas schedules no frames; zoom and pan are O(1) per input event, and painting is a linear scan with a cheap bounds reject. `for_each_point()` visits the document in place through `&mut dyn FnMut`, so no snapshot or per-frame allocation is made. When entity counts grow, a spatial index (grid/quadtree) and a bounds query on the port — or cached tessellation — can replace the linear scan without changing presentation.

Canvas cursor: when the pointer is inside the `CentralPanel`, `adapters/ui/canvas.rs` calls `set_cursor_image()` with the crosshair-with-pickbox bitmap from `adapters/ui/cursor.rs`. The bitmap is embedded as raw RGBA (`include_bytes!`, no runtime image-decoding dependency), cached in a `OnceLock` so egui-winit can dedupe it by `Arc` pointer, and falls back to `egui::CursorIcon::Crosshair` on integrations that cannot display bitmaps. Outside the canvas the custom image is not set, so widgets keep their normal cursor. The Point tool icon is a simple filled dot with a faint ring, distinct from the canvas crosshair.

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

`ViewTransform { pan: Vec2<f64>, zoom: f64, dpi }` converts world→screen with a Y flip. Zoom is around the cursor (keep the world point under the mouse fixed). The current `presentation::canvas::Camera` already implements this mapping (origin + zoom, Y flip) in canvas-local units; the millimetre-based `ViewTransform` will replace it once units land.

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
