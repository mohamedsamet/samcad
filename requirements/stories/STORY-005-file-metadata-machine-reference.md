# STORY-005: File metadata — timestamp + machine reference

> Status: **Planned** · Priority: **P0** · Related: FR-02 (spec requirement)

## Requirement

Every `.samcad` file includes save timestamp and machine identity (hostname + IP).

## Acceptance Criteria

- [ ] `saved_at` field: ISO-8601 UTC timestamp (e.g., `"2026-11-20T15:30:00Z"`) captured at write time.
- [ ] `machine.hostname`: system hostname from `gethostname` crate.
- [ ] `machine.ip`: first non-loopback IPv4 address from `local-ip-address` crate; `"unknown"` if none found.
- [ ] Both fields are present in every saved file.
- [ ] On load, metadata is parsed but not used for logic (informational only).
- [ ] If hostname/IP detection fails, save proceeds with `"unknown"` — never blocks save.
- [ ] `cargo fmt`, `cargo build`, `cargo test` pass.

## Technical Notes

- New module `src/core/machine.rs`:
  ```rust
  pub fn collect_machine_info() -> MachineInfo { ... }
  ```
- `MachineInfo { hostname: String, ip: String }` in `src/core/format.rs`.
- Called from `SaveController` at save time (not at app start, so it reflects the machine that actually saved).
- No UI surface needed; metadata is for forensics/audit.

## Dependencies

- STORY-001 (format structs).
- New deps: `gethostname = "0.4"`, `local-ip-address = "0.3"` in `Cargo.toml`.

## Exit Criteria

Manual test: save file → `cat file.samcad | jq .machine` → shows correct hostname and IP (or "unknown" if offline).