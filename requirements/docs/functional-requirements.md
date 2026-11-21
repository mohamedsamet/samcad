# Functional Requirements

> Status: **Draft** · Last updated: 2026-11-21
> Priority: **P0** = MVP · **P1** = first full release · **P2** = later

## 1. Document & file management

| ID | Priority | Requirement | Acceptance criteria |
| --- | --- | --- | --- |
| FR-01 | P0 | Create a new empty drawing | New drawing opens with default camera (1:1, origin 0,0) and an empty document. If the current drawing has unsaved changes, a confirmation dialog offers **Save** (save via the normal Save flow, then start fresh) or **Don't Save** (discard and start fresh); if clean, the project resets immediately with no dialog |
| FR-02 | P0 | Save/load native `.samcad` project | Drawing round-trips: point geometry and camera view state are preserved. Files are plain, human-readable versioned JSON (v1) — no encryption — matching the specification in `../stories/PLAN-samcad-save-format.md`. Legacy encrypted files are rejected with a clear migration message |
| FR-03 | P0 | Open/save via native file dialogs | Uses the `zenity` file-selection dialog on Linux, filtered to `.samcad`. Works with keyboard shortcuts (Ctrl+O / Ctrl+S / Ctrl+Shift+S). Opening a file while the drawing is dirty asks **Save** / **Don't Save** / **Cancel** before loading |
| FR-04 | P1 | Import DXF (supported subset) | Lines, polylines, circles, arcs, text and layers load with correct coordinates |
| FR-05 | P1 | Export DXF | Exported file opens in LibreCAD/QCAD with geometry and layers intact |
| FR-06 | P1 | Export PDF | Vector output at selectable paper size (A4/A3/Letter) and scale, with optional frame/title block |
| FR-07 | P2 | Export SVG | Vector SVG suitable for web/print workflows |
| FR-08 | P2 | Headless CLI conversion | `samcad convert input.dxf output.pdf` works without a display |
| FR-09 | P0 | "New Project" entry in the File menu | Resets the project immediately when there are no unsaved changes; otherwise opens a confirmation dialog with **Save** (persist via FR-02, then reset) and **Don't Save** (reset without saving) |
| FR-09a | P0 | "Save", "Save As..." and "Export" entries in the File menu | All appear under File. **Save** overwrites the current path silently (first save opens a dialog); **Save As...** always opens a dialog and picks a unique name; **Export** is deferred to FR-05–FR-06 |
| FR-09b | P0 | Point tool in the left palette | Fifth icon button (Point) selects the point tool; highlight/tooltip behave like the other tools |
| FR-09c | P0 | Escape clears the active tool | Pressing Esc deselects the current tool; no palette button is highlighted and clicking the canvas does nothing until a tool is picked again |
| FR-09d | P0 | Dirty tracking and dynamic document title | The drawing tracks unsaved changes (`is_dirty` / `mark_saved`). The main bar shows the current file stem after the File menu with a trailing `*` while dirty; saving, opening or starting a new project clears the flag |
| FR-09e | P0 | Open confirmation when unsaved changes exist | Opening a file with unsaved changes shows **Save** / **Don't Save** / **Cancel**. Save persists the current drawing then loads the selected file; Don't Save loads immediately; Cancel aborts the open |
| FR-09f | P0 | Save As generates a unique filename | When the chosen `.samcad` path already exists, a numeric suffix (` (2)`, ` (3)`, …) is added before the extension so existing files are never overwritten by Save As |

### Save scenarios

The Save/New/Open flows are defined by the drawing's dirty state and whether a file path is known. The following table is the authoritative behavior to verify for FR-01, FR-02, FR-03 and FR-09–FR-09f.

| # | Trigger | State | Behavior |
| --- | --- | --- | --- |
| S1 | `File → Save` / Ctrl+S | Path known | Overwrite the current path silently; mark the drawing clean; no dialog |
| S2 | `File → Save` / Ctrl+S | No path yet | Open the native save dialog; on confirm write the file, set the current path, mark clean; on cancel do nothing |
| S3 | `File → Save As...` / Ctrl+Shift+S | Any | Always open the native save dialog; if the selected path exists, append a numeric suffix before the extension; write the file, update the path, mark clean; on cancel do nothing |
| S4 | `File → New Project` | Clean | Reset drawing, camera and path immediately; no dialog |
| S5 | `File → New Project` | Dirty | Show the **New Project** dialog (**Save** / **Don't Save**). Save persists via S1/S2 and only resets on success; Don't Save resets immediately |
| S6 | `File → Open Project` / Ctrl+O | Clean | Open the native open dialog; on selection load the file, set the path, mark clean |
| S7 | `File → Open Project` / Ctrl+O | Dirty | Show the **Open File** dialog (**Save** / **Don't Save** / **Cancel**). Save persists via S1/S2 then loads; Don't Save loads immediately; Cancel aborts the open |
| S8 | Load failure (missing file, malformed JSON, unsupported version, legacy encrypted file) | Any | Leave the current drawing untouched and report the error; never crash |

### `.samcad` file specification (v1)

The file is UTF-8, pretty-printed JSON with the top-level fields below. The full schema and rationale live in `../stories/PLAN-samcad-save-format.md`.

| Field | Type | Notes |
| --- | --- | --- |
| `samcad_version` | u32 | `1` for this format; any other value is rejected |
| `saved_at` | string | ISO-8601 UTC timestamp of the save |
| `machine.hostname` | string | Host that wrote the file |
| `machine.ip` | string | Primary non-loopback IPv4 address, or `"unknown"` |
| `document.version` | u32 | `1` |
| `document.points` | array of `{ x, y }` | World coordinates (f64, Y-up) |
| `camera.origin_x`, `camera.origin_y` | f64 | World coordinate at canvas top-left |
| `camera.zoom` | f32 | Screen points per world unit (clamped 0.01–10 000) |

Files are intentionally unencrypted so they stay human-readable and portable. Content that looks like the former base64 encrypted blob is rejected with `LegacyEncryptedFormat` and a message directing the user to re-save with a previous SamCAD version. No keyring or encryption dependency is required to save or load.

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
| FR-16a | P0 | Draw a point by clicking | With the Point tool active, clicking in the drawing area stores a point in the document and paints a white 2 px marker at the click position; markers persist while the app runs |
| FR-17 | P2 | Ellipse, spline, hatch, dimensions, leader, block/insert | Each added entity type has create/edit + DXF + PDF support |

## 3. Editing

| ID | Priority | Requirement | Acceptance criteria |
| --- | --- | --- | --- |
| FR-20 | P0 | Select entity | Click to select; rubber-band/window select; shift-click to add to selection |
| FR-21 | P0 | Move selection | Drag or typed offset; preview while dragging |
| FR-22 | P0 | Delete selection | Delete key; selected entity removed |
| FR-23 | P0 | Undo / redo | Ctrl+Z for undo; Ctrl+Y and Ctrl+Shift+Z for redo; covers create/edit/delete and layer changes; integration tests verify shortcuts |
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
| FR-33 | P0 | Snap modes | Grid, endpoint (point entities), midpoint, center, intersection; toggle per mode. Point entity snapping implemented (10px radius, visual indicator) |
| FR-34 | P1 | Orthogonal / polar constraint | Shift constrains angle to 45° increments (or toggle ortho) |
| FR-35 | P1 | Ruler/guides & measured distances | Temporary dimension while drawing |
| FR-36 | P2 | Multiple viewports / named views | Save and recall zoomed views |

Implemented so far: wheel zoom anchored at the cursor (Ctrl+wheel and trackpad pinch also work), clamped to 0.01–10 000 px per world unit; middle-drag pan; and primary-drag pan while no tool is active. Still pending for FR-30: space-drag pan and zoom-to-fit. FR-31 (adaptive grid) is not implemented yet.

File menu is now in the permanent main bar (FR-55). "New Project" and "Open Project" use the generic confirmation dialog system (FR-09 / FR-09e) when there are unsaved changes: **Save** persists through the normal Save flow (FR-02), **Don't Save** discards, and Open additionally offers **Cancel**. Saving produces a plain versioned JSON `.samcad` file (FR-02). Escape closes dialogs; backdrop click closes dialogs. The `FileDialog` port (ADR-009) makes dialog behavior mockable in unit tests so headless test runs never spawn a native dialog.

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
| FR-55 | P0 | Permanent main bar | Always displayed at the top; File menu on the left, the current document stem (with `*` when dirty, see FR-09d) after it, version title in the center, and Minimize / Close controls on the right |
| FR-56 | P0 | File menu in the main bar | Clicking **File** opens a menu with **New Project**, **Open Project**, **Save**, **Save As...** and **Export**; clicking any entry closes the menu. New Project / Open Project may open a confirmation dialog (see FR-09 / FR-09e) |
| FR-57 | P1 | Configuration file (`samcad.env`) | Simple `KEY=VALUE` file in the working directory to parameterize the app: theme colours (snap indicator, etc.), preferences; comments with `#`; missing file falls back to defaults |

## Out of scope (explicit)

- DWG writing, 3D, parametric constraints/solver, CAM, collaboration, plugins in v1.
- Raster image editing, OCR, PDF annotation/signing.
