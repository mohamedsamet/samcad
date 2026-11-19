# Technical Stack — Research & Recommendation

> Status: **Draft** · Last updated: 2026-11-19
> Research note: live web search was unavailable during this pass; findings below come from primary sources fetched directly (crates.io, docs.rs, project GitHub repos). Versions were current at the time of writing.

## Recommendation at a glance

| Concern | Choice | Why |
| --- | --- | --- |
| Language | **Rust (stable)** | Speed, memory safety, single binary, strong typing for geometry/document model |
| GUI framework | **egui + eframe** | Pure Rust, immediate mode, simple, 60 Hz target in debug, native + web/WASM from one codebase |
| Canvas rendering (MVP) | **egui `Painter` / `epaint`** | Built-in anti-aliased lines, shapes and text; no custom engine needed initially |
| Geometry/path math | **lyon** (`lyon_geom`, `lyon_path`, `lyon_tessellation`) | Mature 2D path, curve and tessellation algorithms; already used internally by egui |
| CAD file I/O | **`dxf` (ixmilia/dxf-rs)** | Reads and writes DXF/DXB, MIT licensed, supports R10–R2018, active enough (306 commits, 143 stars) |
| PDF export | **`printpdf`** | Vector lines/shapes/text, layers, paper units, fonts; MIT |
| File dialogs | **`rfd`** | Native dialogs on desktop, WASM support |
| Serialization | **`serde` + `serde_json` or `ron`** | Simple, human-readable native `.samcad` documents and settings |
| CLI (headless) | **`clap`** | Batch conversions and automated tests without a GUI |
| Errors/logging | **`thiserror`, `anyhow`, `tracing`** | Clear error boundaries; core uses typed errors, app uses `anyhow` |

Suggested starting dependencies (pin exact versions in `Cargo.lock`; ranges shown only as a guide):

```toml
eframe   = "0.31"      # also pulls egui, egui-winit, glow/wgpu backend
egui     = "0.31"
lyon     = "1.0"
dxf      = "0.6"
printpdf = "0.12"
rfd      = "0.15"
serde    = { version = "1", features = ["derive"] }
serde_json = "1"
clap     = { version = "4", features = ["derive"] }
thiserror = "2"
anyhow   = "1"
tracing  = "0.1"
tracing-subscriber = "0.3"
```

## Research findings (primary sources)

### egui / eframe — UI layer

- `egui` is an easy-to-use immediate-mode GUI in pure Rust; `eframe` is the official framework for web + native (Linux, macOS, Windows, Android). (<https://github.com/emilk/egui>)
- ~32.4k stars, MIT OR Apache-2.0, no `unsafe` in egui, minimal core dependencies.
- Goals include "responsive: target 60 Hz in debug build" and "a simple 2D graphics API for custom painting (`epaint`)" — exactly what a drawing canvas needs.
- Repaints only on interaction/animation, so an idle editor costs almost no CPU.
- Convention: coordinates in logical points, origin top-left; we will apply our own world→screen transform for CAD coordinates.
- Caveat: egui's APIs are still evolving (breaking changes happen), and complex widget layout is harder in immediate mode. For a canvas-centric app this is a minor concern. Pin versions and upgrade deliberately.

### dxf (ixmilia/dxf-rs) — CAD interchange

- `dxf` 0.6.2, MIT, "A rust crate for reading and writing DXF and DXB CAD files." (<https://docs.rs/dxf/latest/dxf/>, <https://github.com/ixmilia/dxf-rs>)
- Provides `Drawing::load_file` / `save_file`, entities (`Line`, `Circle`, `LwPolyline`, ...), layers, tables, blocks, colors, line weights, headers — i.e. the model we need for import/export.
- Documentation references DXF specs from R10 through R2018; R2014 docs are sufficient for most scenarios.
- Moderate community size (143 stars / 52 forks / 306 commits); acceptable for our needs, and MIT licensed so we can vendor/fork if needed.
- **DWG is not supported and should not be a v1 target** — it is a proprietary format; realistic DWG support means ODA SDK (commercial) or `libredwg` (read-oriented) later.

### printpdf — PDF output

- `printpdf` 0.12.13, MIT, "Rust library for reading and writing PDF files." (<https://docs.rs/printpdf/latest/printpdf/>)
- Has modules for typed units (`Pt`, `Mm`, `Px`), graphics, shapes, text/font handling, colors, layers, SVG and XObjects; built on `lopdf`.
- Good fit for vector CAD output: draw each entity as PDF path operators, not as a raster screenshot.
- Caveat: font embedding/shaping is the fiddly part. Start with a single bundled open font and basic Latin text; improve later.

### lyon — geometry/tessellation

- `lyon` 1.0.19, MIT OR Apache-2.0, "2D Graphics rendering on the GPU using tessellation." (<https://docs.rs/lyon/latest/lyon/>)
- Sub-crates: `lyon_geom` (béziers/arcs), `lyon_path` (path building), `lyon_tessellation` (fill/stroke to triangles), `lyon_algorithms`.
- Useful for filled shapes/hatches and for high-performance rendering later. Not required for the first line/circle MVP since egui already paints these.

### Reference points (what *not* to copy)

- FreeCAD is a full parametric 3D modeler built on OpenCASCADE + Coin3D + Python + Qt, with ~49.5k commits. (<https://github.com/FreeCAD/FreeCAD>)
- That scale confirms the scope decision: samcad should be a focused 2D editor, not a general CAD kernel. Do not introduce a heavy geometry kernel or a retained-mode widget framework for v1.

## Alternatives considered

| Option | Verdict |
| --- | --- |
| TypeScript/Electron or Tauri + Canvas/SVG | Great for web distribution, but heavier runtime and weaker geometry typing; we can still get WASM via egui later |
| `iced` (Rust) | Retained-mode Elm-style architecture; less ergonomic for a custom interactive canvas than egui's direct input/painting |
| Slint / Qt (C++) | Mature widgets, but heavier toolchain/licensing considerations and more boilerplate for a canvas app |
| Custom wgpu engine from day one | Maximum performance, far more code; defer until egui painting is measured to be a bottleneck |
| OpenCASCADE / CAD kernel | Overkill for 2D layouts; introduces C++ toolchain and huge API surface |
| Write DWG directly | Proprietary/legal and technical risk; export DXF instead |

## Key risks & mitigations

| Risk | Mitigation |
| --- | --- |
| DXF spec is large and inconsistent across versions | Implement a small subset first (ASCII R12/R2000, LINE/LWPOLYLINE/CIRCLE/ARC/TEXT/ layers); add entities incrementally with golden-file tests |
| egui breaking API changes | Pin versions, keep UI code thin, isolate canvas painting behind our own traits |
| PDF fonts/text fidelity | Bundle one open font, test text export early, add font selection later |
| Canvas performance with many entities | View-frustum culling + simple LOD; benchmark at 10k/50k entities; escape hatch: lyon + `femtovg`/wgpu renderer |
| DWG expectations from users | Clearly document DXF as the CAD interchange format; consider read-only DWG via `libredwg` in a later phase |
