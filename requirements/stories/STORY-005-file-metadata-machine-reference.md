# STORY-005: File metadata — timestamp + machine reference

> Status: **Done** · Priority: **P0** · Related: FR-02 (file saved specification)

## Requirement

Every `.samcad` file includes save timestamp and machine identity (hostname + IP).

## Acceptance Criteria

- [x] `saved_at` field: ISO-8601 UTC timestamp captured at write time via `time::OffsetDateTime::now_utc()` and RFC 3339 formatting.
- [x] `machine.hostname`: system hostname from `gethostname` crate.
- [x] `machine.ip`: primary address from `local-ip-address` crate; `"unknown"` if detection fails.
- [x] Both fields are present in every saved file (verified by the plain-JSON save test).
- [x] On load, metadata is parsed but not used for logic (informational only).
- [x] If hostname/IP detection fails, save proceeds with `"unknown"` — never blocks save.
- [x] `cargo fmt`, `cargo check`, `cargo build`, `cargo test` pass.

## Technical Notes

- Module `src/core/machine.rs` exposes `collect_machine_info() -> MachineInfo`.
- `MachineInfo { hostname: String, ip: String }` in `src/core/format.rs`.
- Called from `SaveController::build_samcad_file()` at save time (not at app start, so it reflects the machine that actually saved).
- No UI surface needed; metadata is for forensics/audit and is part of the `.samcad` file specification.

## Dependencies

- STORY-001 (format structs).
- New deps: `gethostname = "0.4"`, `local-ip-address = "0.3"` in `Cargo.toml`.

## Exit Criteria

Manual test: save file → `cat file.samcad | jq .machine` → shows correct hostname and IP (or "unknown" if offline).