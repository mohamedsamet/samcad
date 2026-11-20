# Plan: Native `.samcad` Save/Load Format

> Status: **Planned** · Created: 2026-11-20

## Overview

Implement the native `.samcad` file format (versioned JSON via serde) for saving and loading SamCAD drawings. This fulfills FR-02 (Save/load native `.samcad` project), FR-03 (Open/save via native file dialogs), and FR-09a (Save entry in File menu) — all P0 in the MVP.

The format stores:
- File format version (`samcad_version`)
- Save datetime (ISO-8601 UTC)
- Machine reference (hostname + IP address)
- Document data (points, future entities, layers)
- Camera view state (zoom + pan) so the view restores on load

---

## File Format Specification

### JSON Schema (v1)

```json
{
  "samcad_version": 1,
  "saved_at": "2026-11-20T15:30:00Z",
  "machine": {
    "hostname": "workstation-7",
    "ip": "192.168.1.42"
  },
  "document": {
    "version": 1,
    "points": [
      {"x": 12.5, "y": 34.0},
      {"x": -3.5, "y": 4.25}
    ]
  },
  "camera": {
    "origin_x": 0.0,
    "origin_y": 0.0,
    "zoom": 1.0
  }
}
```

### Field Rationale

| Field | Type | Purpose |
|---|---|---|
| `samcad_version` | `u32` | Format version for migration. v1 = this spec. |
| `saved_at` | `String` (ISO-8601 UTC) | When the file was saved. Human-readable, sortable. |
| `machine.hostname` | `String` | Machine name (from `gethostname` or equivalent). |
| `machine.ip` | `String` | Primary IPv4 address (non-loopback) or "unknown". |
| `document.version` | `u32` | Document schema version (separate from file format version). |
| `document.points` | `Array<Point>` | World coordinates (f64, Y-up) — directly from `Document::points()`. |
| `camera.origin_x`, `camera.origin_y` | `f64` | World coordinate at canvas top-left. Matches `Camera::origin`. |
| `camera.zoom` | `f32` | Screen points per world unit. Matches `Camera::zoom` (clamped 0.01–10000). |

**Coordinate system decision:** All geometry is stored in **world coordinates** (f64, Y-up), exactly as held in `Document` and used by the camera. Screen coordinates are never stored — they are a view concern reconstructed by `Camera` on load. This matches the existing architecture (ADR-005, `architecture.md`).

**Camera persistence:** On save, serialize `Camera { origin, zoom }`. On load, reconstruct the camera so the user sees the same zoom/pan. If camera is missing (future-proofing), default to `Camera::default()` (1:1, origin 0,0).

---

## Architecture & Code Placement

Following hexagonal boundaries (`core` ← `application` ← `adapters`):

| Component | Location | Responsibility |
|---|---|---|
| `SamcadFile`, `DocumentData`, `CameraData`, `MachineInfo` | `src/core/format.rs` (new) | Domain structs + serde (de)serialize. No IO deps. |
| `Document::from_data` / `Document::to_data` | `src/core/document.rs` | Convert between domain model and serializable data. |
| `Camera::to_data` / `Camera::from_data` | `src/presentation/camera.rs` (new file or extend `canvas.rs`) | Serialize/deserialize camera state. |
| `SaveUseCase` trait + `SaveController` | `src/application/save.rs` (new) | Driving port: `save(path)`, `save_as()`. Uses driven port `FileSystem`. |
| `LoadUseCase` trait + `LoadController` | `src/application/load.rs` (new) | Driving port: `load(path)`. |
| `FileSystem` trait + `StdFileSystem` | `src/adapters/io/` (new mod) | Driven port: `read(path)`, `write(path, bytes)`. Adapter implements with `std::fs`. |
| File dialog | `src/adapters/ui/file_dialog.rs` (new) | Native open/save dialog via `rfd` or `eframe` native. |
| Menu wiring | `src/adapters/ui/app.rs` + `presentation/main_bar.rs` | Connect Save/Save As/Open to use cases. |

**Dependencies:**
- Add `serde = { version = "1.0", features = ["derive"] }` and `serde_json = "1.0"` to `Cargo.toml`.
- Add `gethostname = "0.4"` for machine name.
- Add `local-ip-address = "0.3"` (or similar) for primary IP detection.

---

## Stories to Create in `requirements/stories/`

Each story follows the format in `requirements/docs/functional-requirements.md` (ID, Priority, Requirement, Acceptance Criteria).

| Story | Title | Priority | Maps to |
|---|---|---|---|
| STORY-001 | Save native `.samcad` file | P0 | FR-02, FR-09a |
| STORY-002 | Load native `.samcad` file | P0 | FR-02, FR-03 |
| STORY-003 | Save As and native file dialogs | P0 | FR-03, FR-09a |
| STORY-004 | Camera view persistence (zoom/pan) | P0 | FR-02 (implied) |
| STORY-005 | File metadata: timestamp + machine info | P0 | FR-02 (spec) |

**Deferred (not in this plan):** Autosave/recovery (Roadmap Phase 1) — per user direction.

---

## Implementation Sequence

### Step 1: Core format structs + serde
- Create `src/core/format.rs` with `SamcadFile`, `DocumentData`, `CameraData`, `MachineInfo`.
- Add `Document::to_data()` / `Document::from_data(data)` in `document.rs`.
- Add `Camera::to_data()` / `Camera::from_data(data)` in `presentation/canvas.rs`.
- Unit tests: round-trip serialize → deserialize equals original.

### Step 2: Machine info capture
- New module `src/core/machine.rs` (or inline in `format.rs`): `fn collect_machine_info() -> MachineInfo`.
- Uses `gethostname` + `local-ip-address` (first non-loopback IPv4).

### Step 3: Application use cases
- `SaveUseCase` trait + `SaveController` in `application/save.rs`.
- `LoadUseCase` trait + `LoadController` in `application/load.rs`.
- Both take `FileSystem` driven port.

### Step 4: Adapter IO
- `src/adapters/io/mod.rs` with `FileSystem` trait + `StdFileSystem` impl.
- `read(path)` → `String` → `serde_json::from_str` → `SamcadFile`.
- `write(path, samcad_file)` → `serde_json::to_string_pretty` → `fs::write`.

### Step 5: File dialog adapter
- `src/adapters/ui/file_dialog.rs` using `rfd` (cross-platform native dialogs) or `eframe`'s built-in.
- `open_file()` → `Option<PathBuf>`, `save_file()` → `Option<PathBuf>`.

### Step 6: Wire into UI
- Extend `WindowUseCase` (or new `FileUseCase`) with `save()`, `save_as()`, `open()`.
- Update `WindowController` to hold `SaveController` + `LoadController`.
- In `adapters/ui/app.rs`: handle Ctrl+S, Ctrl+Shift+S, Ctrl+O; call use cases.
- In `presentation/main_bar.rs`: ensure Save/Save As/Open menu items exist and return actions.

### Step 7: Camera restore on load
- `LoadController` returns both `DocumentData` and `CameraData`.
- `SamcadApp` (or `WindowController`) reconstructs `Camera` from loaded data.

---

## Acceptance Criteria (Aggregate)

- [ ] User can press Ctrl+S → native save dialog → file written as `.samcad` (JSON).
- [ ] File contains: format version, ISO-8601 timestamp, hostname, IP, all points, camera zoom/pan.
- [ ] User can press Ctrl+O → native open dialog → selects `.samcad` → drawing + camera restore exactly.
- [ ] File is human-readable JSON; `cargo fmt` / `cargo build` / `cargo test` pass.
- [ ] Round-trip test: create points, zoom/pan, save, reopen → points + camera identical.
- [ ] Missing/invalid camera in file → defaults to `Camera::default()` (graceful).
- [ ] Unknown `samcad_version` → error dialog, no crash.

---

## Risks & Mitigations

| Risk | Mitigation |
|---|---|
| IP detection fails (no network, multiple interfaces) | Fall back to "unknown"; log warning; don't block save. |
| Future schema changes break old files | `samcad_version` field + match on read; migrate or reject with clear error. |
| Camera origin uses f64 but zoom f32 — precision on huge drawings | Acceptable; zoom is view-only, origin is world. Document coordinates are f64. |
| File dialog blocks on some platforms (Wayland, etc.) | Use `rfd` which handles platform quirks; test on Linux/Windows/macOS. |

---

## Next Steps

1. Copy this plan to `requirements/stories/` (create the dir if needed).
2. Create individual story files STORY-001 through STORY-005 in that dir.
3. Begin implementation with Step 1 (core format structs).