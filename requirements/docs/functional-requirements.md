# Functional Requirements

> Status: **Draft** · Last updated: 2026-11-19
> Priority: **P0** = MVP · **P1** = first full release · **P2** = later

## 1. Document & file management

| ID | Priority | Requirement | Acceptance criteria |
| --- | --- | --- | --- |
| FR-01 | P0 | Create a new empty drawing | New drawing opens with default layer, grid and sensible units (mm) |
| FR-02 | P0 | Save/load native `.samcad` project | Drawing round-trips: geometry, layers, view state and styles are preserved |
| FR-03 | P0 | Open/save via native file dialogs | Works with keyboard shortcuts (Ctrl+O / Ctrl+S / Ctrl+Shift+S) |
| FR-04 | P1 | Import DXF (supported subset) | Lines, polylines, circles, arcs, text and layers load with correct coordinates |
| FR-05 | P1 | Export DXF | Exported file opens in LibreCAD/QCAD with geometry and layers intact |
| FR-06 | P1 | Export PDF | Vector output at selectable paper size (A4/A3/Letter) and scale, with optional frame/title block |
| FR-07 | P2 | Export SVG | Vector SVG suitable for web/print workflows |
| FR-08 | P2 | Headless CLI conversion | `samcad convert input.dxf output.pdf` works without a display |
| FR-09 | P0 | "New Project" entry in the File menu | Selecting it closes the menu; creating the actual empty drawing is deferred to FR-01 |

## 2. Drawing entities

| ID | Priority | Requirement | Acceptance criteria |
| --- | --- | --- | --- |
| FR-10 | P0 | Line | Click-drag or two-click creation; snaps apply |
| FR-11 | P0 | Polyline | Multiple segments, enter/ESC to finish, optional closed flag |
| FR-12 | P0 | Rectangle | Defined by two opposite corners |
| FR-13 | P0 | Circle | Center + radius |
| FR-14 | P0 | Arc | Center, radius, start/end angles (3-point form later) |
| FR-15 | P0 | Text | Single-line text with position, height and rotation |
| FR-16 | P1 | Point construction entity | Visible marker, exportable to DXF |
| FR-17 | P2 | Ellipse, spline, hatch, dimensions, leader, block/insert | Each added entity type has create/edit + DXF + PDF support |

## 3. Editing

| ID | Priority | Requirement | Acceptance criteria |
| --- | --- | --- | --- |
| FR-20 | P0 | Select entity | Click to select; rubber-band/window select; shift-click to add to selection |
| FR-21 | P0 | Move selection | Drag or typed offset; preview while dragging |
| FR-22 | P0 | Delete selection | Delete key; selected entity removed |
| FR-23 | P0 | Undo / redo | Ctrl+Z / Ctrl+Y (or Ctrl+Shift+Z); covers create/edit/delete and layer changes |
| FR-24 | P1 | Edit properties panel | Change coordinates, radius, layer, color, line weight of selection |
| FR-25 | P1 | Copy / repeat-offset | Duplicate selection with a displacement |
| FR-26 | P1 | Trim/extend & fillet | Basic 2D editing operations on lines/arcs |
| FR-27 | P2 | Rotate, scale, mirror, array | Transform selections around a base point |

## 4. Viewport & navigation

| ID | Priority | Requirement | Acceptance criteria |
| --- | --- | --- | --- |
| FR-30 | P0 | Pan and zoom | Mouse wheel zoom around cursor; middle-drag/space-drag pan; zoom-to-fit |
| FR-31 | P0 | Infinite canvas with grid | Grid adaptive to zoom level; toggle visibility |
| FR-32 | P0 | Coordinate readout | Cursor world coordinates shown in status bar |
| FR-33 | P1 | Snap modes | Grid, endpoint, midpoint, center, intersection; toggle per mode |
| FR-34 | P1 | Orthogonal / polar constraint | Shift constrains angle to 45° increments (or toggle ortho) |
| FR-35 | P1 | Ruler/guides & measured distances | Temporary dimension while drawing |
| FR-36 | P2 | Multiple viewports / named views | Save and recall zoomed views |

## 5. Layers & styles

| ID | Priority | Requirement | Acceptance criteria |
| --- | --- | --- | --- |
| FR-40 | P0 | Layer list | Add, rename, delete, set active layer |
| FR-41 | P0 | Layer visibility and lock | Toggle eye/lock; hidden layers are not painted or exported |
| FR-42 | P0 | Layer color and line weight | Applied to all entities on the layer; respected in DXF/PDF |
| FR-43 | P1 | Per-entity style override | Entity may override layer color/line weight |
| FR-44 | P2 | Named styles / dimension styles | Reusable text and dimension settings |

## 6. Usability & platform

| ID | Priority | Requirement | Acceptance criteria |
| --- | --- | --- | --- |
| FR-50 | P0 | Keyboard shortcuts | Common commands have single-key shortcuts (L line, PL polyline, C circle, etc.) and a shortcut cheat sheet |
| FR-51 | P0 | Command/status hint line | Shows current tool and next expected input |
| FR-52 | P1 | Dark/light theme | Follows egui theme with CAD-appropriate colors |
| FR-53 | P1 | Localization-ready strings | UI text centralized; no hardcoded user-facing strings in core |
| FR-54 | P2 | Web (WASM) build | Core editor usable in browser with file open/save via browser APIs |
| FR-55 | P0 | Permanent main bar | Always displayed at the top; contains the version title in the center and Minimize / Close controls on the right |
| FR-55a | P0 | Dynamic bar below the main bar | Hidden by default; floats immediately below the permanent main bar, over the tool palette, and contains only the File menu. Appears when the cursor reaches the bar's height; stays visible while the cursor is over either bar; hides 10 s after the cursor leaves both, or immediately on a click outside the bar |
| FR-56 | P0 | File menu in the top bar | Clicking **File** opens a menu containing **New Project**; clicking **New Project** closes the menu |

## Out of scope (explicit)

- DWG writing, 3D, parametric constraints/solver, CAM, collaboration, plugins in v1.
- Raster image editing, OCR, PDF annotation/signing.
