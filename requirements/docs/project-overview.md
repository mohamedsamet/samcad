# Project Overview — samcad

> Status: **Draft** · Last updated: 2026-11-19

## One-line description

**samcad** is a lightweight, fast 2D CAD application for creating layouts, forms and technical drawings, with export to CAD files (DXF) and PDF.

## Problem

Traditional CAD tools are powerful but heavy: slow startup, complex UIs, large installers, expensive licenses and a feature surface dominated by 3D/parametric modeling. Many users only need to draw clean 2D geometry quickly and hand it off as a `.dxf` or a printable `.pdf`.

## Vision

A small, dependable drawing tool that starts instantly, stays out of the way, and produces standards-compliant 2D output that opens correctly in AutoCAD, LibreCAD, QCAD and other CAD tools.

## Target users

- Architects, engineers and technicians who need quick 2D sketches and layouts.
- Users who need to view/edit DXF without a full CAD suite.
- Small teams that want a portable, offline-first drawing tool with simple file interchange.

## Goals

1. **Simple** — a focused 2D tool; a new user can draw a useful layout in minutes.
2. **Very fast** — instant startup, responsive pan/zoom/edit even on large drawings.
3. **Interoperable** — first-class DXF read/write and clean vector PDF output.
4. **Portable** — desktop first (Linux/macOS/Windows), with a web/WASM path preserved by the chosen stack.
5. **Maintainable** — small, testable codebase with a UI-independent core.

## Non-goals (for the first releases)

- Full AutoCAD feature parity or DWG **write** support (DWG is proprietary).
- 3D modeling, parametric solids, FEM/CAM or BIM (that is FreeCAD/Revit territory).
- Cloud collaboration, accounts or telemetry.
- A large plugin ecosystem in v1.

## Initial scope (MVP)

- Drawing primitives: line, polyline, rectangle, circle, arc, text.
- Layers with color/line-weight/visibility.
- Viewport: pan, zoom, grid, infinite canvas.
- Editing: select, move, delete, undo/redo, property editing.
- Files: native project save/load, DXF import/export, PDF export.
- Basic precision: snapping to grid/endpoints and orthogonal constraint.

## Later scope

Dimensions and leaders, hatches, blocks/reusable symbols, dimension styles, plotting/title blocks, SVG export, headless CLI batch conversion, DWG **read** via a third-party library, WASM web build.

## Definition of success for v1

A user can draw a 2D floor-plan-like layout, save it, reopen it, export it to DXF that opens in LibreCAD/QCAD, export a printable PDF at a chosen paper size, and do all of that without a manual or a network connection.
