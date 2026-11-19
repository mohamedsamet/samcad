# samcad

Lightweight 2D CAD application — desktop UI foundation.

The current step is intentionally minimal: a full-screen, frameless window with two top bars. The **main bar** is always visible and holds the centered **SamCad vX.Y.Z** title plus **Minimize** `−` / **Close** `×` controls. Just below it sits a **dynamic bar** that floats over the tool palette and contains only a **File** menu (**New Project**, **Save**, **Export**) with enlarged buttons and text; it is hidden until the cursor reaches the bar's height, stays while the cursor is over either bar, hides again 10 s after the cursor leaves, and can be dismissed immediately by clicking outside it. The left tool palette has five equally sized 48×48 SVG icon buttons (**Point**, **Line**, **Circle**, **Rectangle**, **Arc**) with no "Tools" heading — the tool name appears on hover and the selected button is highlighted. The working area shows a CAD-style bitmap crosshair cursor (extracted from `pointer.jpeg`: full crosshair with a small pickbox at its centre); elsewhere widgets keep their normal cursor. No drawing geometry yet — the buttons only select a tool — and no document model or database.

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

A full-screen window opens with an empty drawing area. The top bar with the version title and the `−` / `×` window controls is always visible. Move the cursor up to the height of the second, dynamic bar just below the main bar — it floats over the tool palette and contains the enlarged **File** menu (**New Project**, **Save**, **Export** — each entry currently just closes the menu). It hides again 10 seconds after the cursor leaves both bars, or immediately if you click outside it. `Alt+F4` / `Ctrl+Q` also work, depending on your desktop environment.

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
cargo test           # run unit tests (tool state, palette, both bars, auto-hide + menu)
cargo fmt --check    # verify formatting
cargo clippy         # lint the code
```

## Architecture

The code follows **hexagonal architecture** (ports and adapters) inside a single crate. Dependencies point inwards only: `application` and `presentation` depend on `core`, and `adapters` depend on `core` and `presentation`.

```text
adapters/ui (egui)
   │  ├──reads──> presentation (egui-agnostic view models / components)
   │  └──calls──> ToolUseCase / WindowUseCase      (driving ports)
   │                        │ implemented by
   │ paints                 ▼
   ▼               ToolController / WindowController
egui widgets             │ calls
                         ▼
                 WindowActions (driven port)
                         │ implemented by
                         ▼
              WindowActionAdapter + Bridge → egui ViewportCommand
```

- `core/` — domain model (`Tool`) and port traits. No framework or I/O dependencies.
- `application/` — use cases implementing the driving ports.
- `presentation/` — UI/product decisions as egui-agnostic values: the palette `Component` iterator, the permanent main bar view model (title + window controls), the dynamic bar view model (File menu), and the auto-hide state machine (reveal at bar height, 10 s hide delay, click-outside dismissal). It consumes plain data (e.g. the selected `Tool`), not application ports, and its view models are static, so painting allocates nothing. All testable without a window.
- `adapters/ui/` — a thin egui renderer: it receives those view models plus a `Theme` (sizes, icon assets) and paints them as a permanent `Panel::top`, a foreground `Area` below it, and palette widgets. All egui-specific parametrization lives here.
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
│   │   ├── component.rs           # Component, IconId, MenuAction, MenuItem, WindowButton
│   │   ├── main_bar.rs            # permanent bar: title + window controls
│   │   ├── palette.rs             # builds the palette component tree + tests
│   │   └── top_bar.rs             # auto-hide state machine + File menu view model
│   └── adapters/
│       ├── mod.rs
│       ├── window.rs              # driven adapter + egui bridge
│       └── ui/
│           ├── mod.rs
│           ├── app.rs             # SamcadApp + native_options()
│           ├── canvas.rs          # central drawing area + CAD cursor
│           ├── theme.rs           # sizes + IconId -> SVG asset mapping
│           ├── cursor.rs          # embedded bitmap crosshair cursor for the canvas
│           ├── main_bar.rs         # permanent Panel::top with title + controls
│           ├── top_bar.rs         # floating Area below it with File menu
│           ├── tool_panel.rs      # renders presentation components with egui
│           └── icons/             # point.svg, line.svg, circle.svg, rectangle.svg, arc.svg,
│                                  # cursor_crosshair.png/.rgba, pointer.jpeg (source)
├── requirements/docs/             # product & engineering documentation
├── README.md                      # this guide
└── .gitignore                     # ignores build output in target/
```

## What's next

The next steps come from [`requirements/docs/roadmap.md`](requirements/docs/roadmap.md): add the drawing canvas (pan/zoom/grid), then the document model, tools, and DXF/PDF export. Cross-platform packaging (Linux + Windows) is deliberately deferred.
