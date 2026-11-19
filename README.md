# samcad

Lightweight 2D CAD application — desktop UI foundation.

The current step is intentionally minimal: a full-screen, frameless window with a centered **SamCad vX.Y.Z** title, **Minimize** (`−`) and **Close** (`×`) buttons on the right, and a left tool palette with four equally sized buttons showing SVG icons (**Line**, **Circle**, **Rectangle**, **Arc**) — the tool name appears on hover and the selected button is highlighted. No drawing geometry yet — the buttons only select a tool — and no document model or database.

Product requirements and stack research live in [`requirements/docs/`](requirements/docs/README.md).

## Prerequisites

- **Rust (stable)** — edition 2024 requires Rust 1.85 or newer. This machine has Rust 1.100.0.
- **Linux desktop** (X11 or Wayland) with OpenGL support. This first UI is built and tested on Linux only.

If Rust is missing on another machine, install it from <https://rustup.rs>:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Check your installation:

```bash
rustc --version
cargo --version
```

## Run it

From the project root:

```bash
cargo run
```

A full-screen window opens with an empty drawing area. Use the `−` and `×` buttons in the top-right corner, or `Alt+F4` / `Ctrl+Q` depending on your desktop environment, to control the window.

To build and run the optimized binary directly:

```bash
cargo build --release
./target/release/samcad
```

## Dependencies

Two direct dependencies:
- [`eframe`](https://crates.io/crates/eframe) 0.37 — provides `egui` and native window integration. Minimize/close use `egui::ViewportCommand`, so no direct `winit` dependency is needed.
- [`egui_extras`](https://crates.io/crates/egui_extras) 0.37 with the `svg` feature — rasterizes the tool-bar SVG icons (via `resvg`). Its image loaders are installed once when the app is created, inside the UI adapter.

## Other useful commands

```bash
cargo test           # run unit tests (tool state + palette components)
cargo fmt --check    # verify formatting
cargo clippy         # lint the code
```

## Architecture

The code follows **hexagonal architecture** (ports and adapters) inside a single crate. Dependencies point inwards only: `adapters` → `presentation` → `application` → `core`.

```text
adapters/ui (egui) ──reads──> presentation ──calls──> ToolUseCase / WindowUseCase
     │                            │                        │ implemented by
     │ paints                     │ builds                  ▼
     ▼                            ▼              ToolController / WindowController
  egui widgets            Component tree (no egui)          │ calls
                                                            ▼
                                                     WindowActions (driven port)
                                                            │ implemented by
                                                            ▼
                                                  WindowActionAdapter + Bridge
                                                            │
                                                            ▼
                                                  egui ViewportCommand
```

- `core/` — domain model (`Tool`) and port traits. No framework or I/O dependencies.
- `application/` — use cases implementing the driving ports.
- `presentation/` — UI/product decisions as egui-agnostic `Component` values: which buttons exist, their tooltips/icons, which is selected. Testable without a window.
- `adapters/ui/` — a thin egui renderer: it receives the component tree plus a `Theme` (sizes, icon assets) and paints it. All egui-specific parametrization lives here.
- `adapters/window.rs` — the "driven" adapter plus the bridge that applies commands to the egui context.
- `src/main.rs` — composition root only: wires the layers and calls `run_native`. No business logic.

## Project structure

```text
samcad/
├── Cargo.toml                     # package metadata + eframe / egui_extras
├── src/
│   ├── main.rs                    # composition root (no business logic)
│   ├── lib.rs                     # crate root: core / application / presentation / adapters
│   ├── core/
│   │   ├── mod.rs
│   │   ├── model.rs               # Tool enum
│   │   └── ports.rs               # WindowUseCase, ToolUseCase, WindowActions
│   ├── application/
│   │   ├── mod.rs
│   │   ├── window.rs              # WindowController use case
│   │   └── tool.rs                # ToolController use case
│   ├── presentation/
│   │   ├── mod.rs
│   │   ├── component.rs           # Component, IconId, Emphasis (no egui)
│   │   └── palette.rs             # builds the palette component tree + tests
│   └── adapters/
│       ├── mod.rs
│       ├── window.rs              # driven adapter + egui bridge
│       └── ui/
│           ├── mod.rs
│           ├── app.rs             # SamcadApp + native_options()
│           ├── theme.rs           # sizes + IconId -> SVG asset mapping
│           ├── title_bar.rs       # top panel, version, minimize/close
│           ├── tool_panel.rs      # renders presentation components with egui
│           └── icons/             # line.svg, circle.svg, rectangle.svg, arc.svg
├── requirements/docs/             # product & engineering documentation
├── README.md                      # this guide
└── .gitignore                     # ignores build output in target/
```

## What's next

The next steps come from [`requirements/docs/roadmap.md`](requirements/docs/roadmap.md): add the drawing canvas (pan/zoom/grid), then the document model, tools, and DXF/PDF export. Cross-platform packaging (Linux + Windows) is deliberately deferred.
